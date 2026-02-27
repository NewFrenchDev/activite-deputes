use gloo_net::http::Request;
use crate::models::*;
use crate::utils::app_base_path;
use std::fmt;

/// Erreurs structurées de l'API
#[derive(Debug, Clone)]
pub enum ApiError {
    /// Erreur réseau (pas de connexion, timeout, etc)
    NetworkError(String),
    
    /// Ressource non trouvée (404)
    NotFound(String),
    
    /// Erreur serveur (5xx)
    ServerError(u16, String),
    
    /// Erreur de parsing JSON
    ParseError(String),
    
    /// Erreur générique
    Other(String),
}

/// Permet d'afficher l'erreur avec Display
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "Erreur réseau: {}", msg),
            ApiError::NotFound(what) => write!(f, "Non trouvé: {}", what),
            ApiError::ServerError(code, msg) => write!(f, "Erreur serveur {}: {}", code, msg),
            ApiError::ParseError(msg) => write!(f, "Erreur parsing JSON: {}", msg),
            ApiError::Other(msg) => write!(f, "Erreur: {}", msg),
        }
    }
}

/// Permet de convertir ApiError en String
impl From<ApiError> for String {
    fn from(err: ApiError) -> Self {
        err.to_string()
    }
}

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

// ============= NOUVELLES FONCTIONS V2 (avec ApiError) =============

pub async fn fetch_status_v2() -> Result<Status, ApiError> {
    let url = format!("{}/data/status.json", base_url());
    let resp = Request::get(&url)
        .send().await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;
    
    let code = resp.status() as u16;
    match code {
        404 => Err(ApiError::NotFound("status.json".to_string())),
        code if code >= 500 => Err(ApiError::ServerError(code, "HTTP error".to_string())),
        code if code >= 400 => Err(ApiError::ServerError(code, format!("HTTP {}", code))),
        _ => resp.json::<Status>()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
}

pub async fn fetch_stats_v2(period: Period) -> Result<Vec<DeputeStats>, ApiError> {
    let url = format!("{}/{}", base_url(), period.json_file());
    let resp = Request::get(&url)
        .send().await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;
    
    let code = resp.status() as u16;
    match code {
        404 => Err(ApiError::NotFound(period.json_file().to_string())),
        code if code >= 500 => Err(ApiError::ServerError(code, "HTTP error".to_string())),
        code if code >= 400 => Err(ApiError::ServerError(code, format!("HTTP {}", code))),
        _ => resp.json::<Vec<DeputeStats>>()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
}

pub async fn fetch_deputes_v2() -> Result<Vec<DeputeInfo>, ApiError> {
    // Essayer d'abord le format chunké (deputes_p1.json, deputes_p2.json, …)
    let mut all: Vec<DeputeInfo> = Vec::new();
    let mut page = 1usize;
    loop {
        let url = format!("{}/data/deputes_p{}.json", base_url(), page);
        let resp = Request::get(&url)
            .send().await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        let code = resp.status() as u16;
        match code {
            // 404 = fin des chunks (ou format chunké absent)
            404 => break,
            code if code >= 500 => return Err(ApiError::ServerError(code, "HTTP error".to_string())),
            code if code >= 400 => return Err(ApiError::ServerError(code, format!("HTTP {}", code))),
            _ => {
                match resp.json::<Vec<DeputeInfo>>().await {
                    Ok(chunk) => {
                        if chunk.is_empty() {
                            break;
                        }
                        all.extend(chunk);
                        page += 1;
                    }
                    Err(_) if page == 1 => {
                        // First chunk parse failed (likely HTML from SPA fallback);
                        // chunked format unavailable, fall through to single-file.
                        break;
                    }
                    Err(e) => {
                        return Err(ApiError::ParseError(e.to_string()));
                    }
                }
            }
        }
    }
    if !all.is_empty() {
        return Ok(all);
    }

    // Fallback : fichier unique deputes.json (ancien format)
    let url = format!("{}/data/deputes.json", base_url());
    let resp = Request::get(&url)
        .send().await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;

    let code = resp.status() as u16;
    match code {
        404 => Err(ApiError::NotFound("deputes.json".to_string())),
        code if code >= 500 => Err(ApiError::ServerError(code, "HTTP error".to_string())),
        code if code >= 400 => Err(ApiError::ServerError(code, format!("HTTP {}", code))),
        _ => resp.json::<Vec<DeputeInfo>>()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
}

pub async fn fetch_group_ppl_index_v2() -> Result<GroupPplIndex, ApiError> {
    let url = format!("{}/data/positions-groupes/ppl/index.json", base_url());
    let resp = Request::get(&url)
        .send().await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;
    
    let code = resp.status() as u16;
    match code {
        404 => Err(ApiError::NotFound("positions-groupes/ppl/index.json".to_string())),
        code if code >= 500 => Err(ApiError::ServerError(code, "HTTP error".to_string())),
        code if code >= 400 => Err(ApiError::ServerError(code, format!("HTTP {}", code))),
        _ => resp.json::<GroupPplIndex>()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
}

pub async fn fetch_group_ppl_group_shard_v2(rel_file: &str) -> Result<GroupPplGroupShard, ApiError> {
    let clean = rel_file
        .trim_start_matches('/')
        .trim_start_matches("data/positions-groupes/ppl/");
    let url = format!("{}/data/positions-groupes/ppl/{}", base_url(), clean);
    let resp = Request::get(&url)
        .send().await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;
    
    let code = resp.status() as u16;
    match code {
        404 => Err(ApiError::NotFound(format!("positions-groupes/ppl/{}", clean))),
        code if code >= 500 => Err(ApiError::ServerError(code, "HTTP error".to_string())),
        code if code >= 400 => Err(ApiError::ServerError(code, format!("HTTP {}", code))),
        _ => resp.json::<GroupPplGroupShard>()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
}

pub async fn fetch_deputy_ppl_shard_v2(deputy_id: &str) -> Result<Option<DeputyPplShard>, ApiError> {
    let file = safe_file_stem_client(deputy_id);
    if file.is_empty() {
        return Ok(None);
    }
    let url = format!("{}/data/positions-deputes/ppl/{}.json", base_url(), file);
    let resp = Request::get(&url)
        .send().await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;
    
    let code = resp.status() as u16;
    match code {
        404 => Ok(None),  // 404 = ok, juste pas de données
        code if code >= 500 => Err(ApiError::ServerError(code, "HTTP error".to_string())),
        code if code >= 400 => Err(ApiError::ServerError(code, format!("HTTP {}", code))),
        _ => resp.json::<DeputyPplShard>()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
            .map(Some)
    }
}

// ============= ANCIENNES FONCTIONS (pour compatibilité) =============
// Ces fonctions appellent les v2 et convertissent ApiError en String
// Aucun code existant ne doit changer

pub async fn fetch_status() -> Result<Status, String> {
    fetch_status_v2().await.map_err(|e| e.to_string())
}

pub async fn fetch_stats(period: Period) -> Result<Vec<DeputeStats>, String> {
    fetch_stats_v2(period).await.map_err(|e| e.to_string())
}

pub async fn fetch_deputes() -> Result<Vec<DeputeInfo>, String> {
    fetch_deputes_v2().await.map_err(|e| e.to_string())
}

pub async fn fetch_group_ppl_index() -> Result<GroupPplIndex, String> {
    fetch_group_ppl_index_v2().await.map_err(|e| e.to_string())
}

pub async fn fetch_group_ppl_group_shard(rel_file: &str) -> Result<GroupPplGroupShard, String> {
    fetch_group_ppl_group_shard_v2(rel_file).await.map_err(|e| e.to_string())
}

pub async fn fetch_deputy_ppl_shard(deputy_id: &str) -> Result<Option<DeputyPplShard>, String> {
    fetch_deputy_ppl_shard_v2(deputy_id).await.map_err(|e| e.to_string())
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
