use gloo_net::http::Request;
use crate::models::*;
use crate::utils::app_base_path;

/// Retourne le chemin de base du site, en gérant correctement GitHub Pages
/// où le site peut être servi depuis /activite-deputes/ et non depuis /.
/// On se base sur window.location.pathname pour détecter le sous-chemin.
pub fn base_url() -> String {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return String::new(),
    };

    let origin = window.location().origin().unwrap_or_default();
    let base_path = app_base_path();

    if base_path == "/" {
        origin
    } else {
        format!("{origin}{base_path}")
    }
}

pub fn inferred_github_repo_urls() -> Option<(String, String)> {
    let window = web_sys::window()?;
    let location = window.location();
    let hostname = location.hostname().ok()?;
    if !hostname.ends_with(".github.io") {
        return None;
    }

    let owner = hostname.trim_end_matches(".github.io").trim();
    if owner.is_empty() {
        return None;
    }

    let pathname = location.pathname().ok()?;
    let first_segment = pathname
        .trim_start_matches('/')
        .split('/')
        .next()
        .unwrap_or("")
        .trim();

    if first_segment.is_empty() || matches!(
        first_segment,
        "depute" | "comparer" | "exporter" | "methodologie" | "stats-globales" | "reseau" | "positions-groupes" | "index.html"
    ) {
        return None;
    }

    let repo_url = format!("https://github.com/{owner}/{first_segment}");
    let issue_url = format!("{repo_url}/issues/new");
    Some((repo_url, issue_url))
}

pub async fn fetch_status() -> Result<Status, String> {
    let url = format!("{}/data/status.json", base_url());
    let resp = Request::get(&url)
        .send().await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<Status>().await.map_err(|e| e.to_string())
}

pub async fn fetch_stats(period: Period) -> Result<Vec<DeputeStats>, String> {
    let url = format!("{}/{}", base_url(), period.json_file());
    let resp = Request::get(&url)
        .send().await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {} pour {}", resp.status(), period.json_file()));
    }
    resp.json::<Vec<DeputeStats>>().await.map_err(|e| e.to_string())
}

pub async fn fetch_deputes() -> Result<Vec<DeputeInfo>, String> {
    let url = format!("{}/data/deputes.json", base_url());
    let resp = Request::get(&url)
        .send().await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<Vec<DeputeInfo>>().await.map_err(|e| e.to_string())
}



pub async fn fetch_group_ppl_index() -> Result<GroupPplIndex, String> {
    let url = format!("{}/data/positions-groupes/ppl/index.json", base_url());
    let resp = Request::get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<GroupPplIndex>().await.map_err(|e| e.to_string())
}

pub async fn fetch_group_ppl_group_shard(rel_file: &str) -> Result<GroupPplGroupShard, String> {
    let clean = rel_file
        .trim_start_matches('/')
        .trim_start_matches("data/positions-groupes/ppl/");
    let url = format!("{}/data/positions-groupes/ppl/{}", base_url(), clean);
    let resp = Request::get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<GroupPplGroupShard>().await.map_err(|e| e.to_string())
}


pub async fn fetch_deputy_ppl_shard(deputy_id: &str) -> Result<Option<DeputyPplShard>, String> {
    let file = safe_file_stem_client(deputy_id);
    if file.is_empty() {
        return Ok(None);
    }
    let url = format!("{}/data/positions-deputes/ppl/{}.json", base_url(), file);
    let resp = Request::get(&url).send().await.map_err(|e| e.to_string())?;
    if resp.status() == 404 {
        return Ok(None);
    }
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let shard = resp.json::<DeputyPplShard>().await.map_err(|e| e.to_string())?;
    Ok(Some(shard))
}

fn safe_file_stem_client(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut prev_dash = false;
    for ch in raw.chars() {
        let c = ch.to_ascii_lowercase();
        let is_alnum = c.is_ascii_alphanumeric();
        if is_alnum {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// Génère un CSV depuis les stats en mémoire (évite la dépendance aux fichiers CSV statiques sur mobile)
pub fn stats_to_csv(stats: &[DeputeStats]) -> String {
    let mut out = String::with_capacity(stats.len() * 200);
    out.push_str("deputy_id,nom,prenom,groupe_abrev,groupe_nom,parti_rattachement,dept,circo,period_start,period_end,scrutins_eligibles,votes_exprimes,non_votant,absent,participation_rate,pour_count,contre_count,abst_count,amd_authored,amd_adopted,amd_adoption_rate,amd_cosigned,interventions_count,interventions_chars,top_dossier_id,top_dossier_titre,top_dossier_score\n");
    for s in stats {
        let top = s.top_dossiers.first();
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.4},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            s.deputy_id,
            csv_escape(&s.nom), csv_escape(&s.prenom),
            csv_opt(&s.groupe_abrev), csv_opt(&s.groupe_nom),
            csv_opt(&s.parti_rattachement),
            csv_opt(&s.dept), csv_opt(&s.circo),
            s.period_start, s.period_end,
            s.scrutins_eligibles, s.votes_exprimes,
            s.non_votant, s.absent,
            s.participation_rate,
            s.pour_count, s.contre_count, s.abst_count,
            s.amd_authored, s.amd_adopted,
            s.amd_adoption_rate.map(|r| format!("{r:.4}")).unwrap_or_default(),
            s.amd_cosigned,
            s.interventions_count, s.interventions_chars,
            top.map(|t| t.dossier_id.as_str()).unwrap_or(""),
            top.map(|t| csv_escape(&t.titre)).unwrap_or_default(),
            top.map(|t| t.score.to_string()).unwrap_or_default(),
        ));
    }
    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn csv_opt(o: &Option<String>) -> String {
    o.as_deref().map(csv_escape).unwrap_or_default()
}
