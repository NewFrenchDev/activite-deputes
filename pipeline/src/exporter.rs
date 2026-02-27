use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use csv::Writer;

use crate::aggregator::AllAggregates;
use crate::downloader::EtagInfo;
use crate::group_ppl_v1;
use crate::models::DeputeStats;

pub fn write_json(
    agg: &AllAggregates,
    temp_dir: &Path,
    etags: &[EtagInfo],
    now: DateTime<Utc>,
) -> Result<()> {
    let data_dir = temp_dir.join("data");
    std::fs::create_dir_all(&data_dir)?;

    // status.json
    let sources: Vec<serde_json::Value> = etags.iter().map(|e| json!({
        "key": e.key,
        "etag": e.etag,
        "last_modified": e.last_modified,
        "size_bytes": e.size_bytes,
    })).collect();

    let status = json!({
        "last_update": now.to_rfc3339(),
        "last_update_readable": now.format("%d/%m/%Y à %H:%M UTC").to_string(),
        "legislature": 17,
        "sources": sources,
        "counts": {
            "deputes": agg.deputes.len(),
        }
    });
    write_json_file(&data_dir.join("status.json"), &status)?;

    // deputes_P30.json
    write_json_file(&data_dir.join("deputes_P30.json"), &json!(agg.p30))?;
    write_json_file(&data_dir.join("deputes_P180.json"), &json!(agg.p180))?;
    write_json_file(&data_dir.join("deputes_LEG.json"), &json!(agg.leg))?;

    // deputes_pN.json — info de base pour le listing, découpée en chunks de 200
    let deputes_base: Vec<serde_json::Value> = agg.deputes.iter().map(|d| json!({
        "id": d.id,
        "nom": d.nom,
        "prenom": d.prenom,
        "date_naissance": d.date_naissance,
        "sexe": d.sexe,
        "pays_naissance": d.pays_naissance,
        "profession": d.profession,
        "groupe_abrev": d.groupe_abrev,
        "groupe_nom": d.groupe_nom,
        "parti_nom": d.parti_nom,
        "dept_code": d.dept_code,
        "dept_nom": d.dept_nom,
        "circo": d.circo,
        "mandat_debut": d.mandat_debut,
        "mandat_fin": d.mandat_fin,
        "mandat_debut_legislature": d.mandat_debut_legislature,
        "mandat_assemblee_episodes": d.mandat_assemblee_episodes,
        "mandat_assemblee_episode_count": d.mandat_assemblee_episodes.len(),
        "mandat_assemblee_episode_labels": d.mandat_assemblee_episodes.iter().enumerate().map(|(i, e)| {
            let fin = e.date_fin.map(|x| x.to_string()).unwrap_or_else(|| "en cours".to_string());
            format!("Mandat AN épisode {}: {} → {}", i + 1, e.date_debut, fin)
        }).collect::<Vec<String>>(),
        "email_assemblee": d.email_assemblee,
        "site_web": d.site_web,
        "sites_web": d.sites_web,
        "sites_web_sources": d.sites_web_sources,
        "telephones": d.telephones,
        "uri_hatvp": d.uri_hatvp,
    })).collect();
    // Fichier unique pour compatibilité (stats_globales, anciens clients, etc.)
    write_json_file(&data_dir.join("deputes.json"), &json!(deputes_base))?;

    // Chunks paginés deputes_p1.json, deputes_p2.json, …
    const DEPUTES_CHUNK_SIZE: usize = 200;
    let chunk_count = (deputes_base.len() + DEPUTES_CHUNK_SIZE - 1) / DEPUTES_CHUNK_SIZE;
    for (i, chunk) in deputes_base.chunks(DEPUTES_CHUNK_SIZE).enumerate() {
        let filename = format!("deputes_p{}.json", i + 1);
        write_json_file(&data_dir.join(&filename), &json!(chunk))?;
    }
    eprintln!("[exporter] deputes.json + {} chunk(s) de {} (total: {} députés)", chunk_count, DEPUTES_CHUNK_SIZE, deputes_base.len());

    // positions-groupes / PPL (V1) — shards par groupe pour limiter la bande passante
    group_ppl_v1::write_group_ppl_json(&agg.deputes, &agg.dossiers, &data_dir, &now.to_rfc3339())?;

    // dossiers_min.json — mapping id -> titre (utilisé par la page Amendements)
    write_dossiers_min_json(&data_dir, &agg.dossiers)?;

    // amendements/ — calendrier jour-par-jour (shards par mois)
    write_amendements_calendar_json(&data_dir, agg, &now.to_rfc3339())?;

    Ok(())
}

#[derive(Debug, Clone, Serialize)]
struct AmdEvent {
    /// Type d'évènement: DEPOT | CIRCULATION | EXAMEN | SORT
    t: &'static str,
    /// ID amendement
    id: String,
    /// Numéro (si présent)
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<String>,
    /// ID auteur (député)
    #[serde(skip_serializing_if = "Option::is_none")]
    aid: Option<String>,
    /// Type d'auteur (Député, Groupe, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    aty: Option<String>,
    /// Cosignataires IDs
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    cos: Vec<String>,
    /// ID dossier
    #[serde(skip_serializing_if = "Option::is_none")]
    did: Option<String>,
    /// Article (ex: "Art. 3")
    #[serde(skip_serializing_if = "Option::is_none")]
    art: Option<String>,
    /// Libellé de sort (uniquement pour t=SORT)
    #[serde(skip_serializing_if = "Option::is_none")]
    s: Option<String>,
    /// true si adopté (uniquement utile pour t=SORT)
    #[serde(default)]
    ok: bool,
    /// Mission visée
    #[serde(skip_serializing_if = "Option::is_none")]
    mis: Option<String>,
    /// Exposé sommaire
    #[serde(skip_serializing_if = "Option::is_none")]
    exp: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct MonthMeta {
    month: String,
    days: usize,
    events: usize,
}

#[derive(Debug, Clone, Serialize)]
struct AmendementsIndex {
    schema_version: u32,
    generated_at: String,
    months: Vec<MonthMeta>,
    undated_count: usize,
    undated_file: String,
    notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct AmendementsMonthFile {
    schema_version: u32,
    month: String,
    /// days[YYYY-MM-DD] = [events...]
    days: BTreeMap<String, Vec<AmdEvent>>,
}

#[derive(Debug, Clone, Serialize)]
struct MonthStat {
    events: usize,
    days: HashSet<String>,
}

fn write_dossiers_min_json(data_dir: &Path, dossiers: &HashMap<String, crate::models::Dossier>) -> Result<()> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for (id, d) in dossiers {
        out.insert(id.clone(), d.titre.clone());
    }
    write_json_file(&data_dir.join("dossiers_min.json"), &json!(out))?;
    Ok(())
}

fn write_amendements_calendar_json(data_dir: &Path, agg: &AllAggregates, generated_at: &str) -> Result<()> {
    let amd_dir = data_dir.join("amendements");
    let months_dir = amd_dir.join("months");
    std::fs::create_dir_all(&months_dir)?;

    // 1) Construire les évènements datés (vector triable) + liste sans date
    //    Note: on n'utilise ici QUE les dates structurées (dateDepot/dateExamen/dateSort/dateCirculation).
    //    Si aucune de ces dates n'est présente, l'amendement est compté "sans date".
    let mut dated: Vec<(NaiveDate, u8, AmdEvent)> = Vec::new();
    let mut undated: Vec<serde_json::Value> = Vec::new();

    for a in &agg.amendements {
        let mut has_any = false;

        // Dépôt
        if let Some(d) = a.date_depot {
            has_any = true;
            dated.push((d, 0, AmdEvent {
                t: "DEPOT",
                id: a.id.clone(),
                n: a.numero.clone(),
                aid: a.auteur_id.clone(),
                aty: a.auteur_type.clone(),
                cos: a.cosignataires_ids.clone(),
                did: a.dossier_ref.clone(),
                art: a.article.clone(),
                s: None,
                ok: false,
                mis: a.mission_visee.clone(),
                exp: a.expose_sommaire.clone(),
            }));
        }

        // Circulation
        if let Some(d) = a.date_circulation {
            has_any = true;
            dated.push((d, 1, AmdEvent {
                t: "CIRCULATION",
                id: a.id.clone(),
                n: a.numero.clone(),
                aid: a.auteur_id.clone(),
                aty: a.auteur_type.clone(),
                cos: a.cosignataires_ids.clone(),
                did: a.dossier_ref.clone(),
                art: a.article.clone(),
                s: None,
                ok: false,
                mis: a.mission_visee.clone(),
                exp: a.expose_sommaire.clone(),
            }));
        }

        // Examen
        if let Some(d) = a.date_examen {
            has_any = true;
            dated.push((d, 2, AmdEvent {
                t: "EXAMEN",
                id: a.id.clone(),
                n: a.numero.clone(),
                aid: a.auteur_id.clone(),
                aty: a.auteur_type.clone(),
                cos: a.cosignataires_ids.clone(),
                did: a.dossier_ref.clone(),
                art: a.article.clone(),
                s: None,
                ok: false,
                mis: a.mission_visee.clone(),
                exp: a.expose_sommaire.clone(),
            }));
        }

        // Sort
        if let Some(d) = a.date_sort {
            has_any = true;
            dated.push((d, 3, AmdEvent {
                t: "SORT",
                id: a.id.clone(),
                n: a.numero.clone(),
                aid: a.auteur_id.clone(),
                aty: a.auteur_type.clone(),
                cos: a.cosignataires_ids.clone(),
                did: a.dossier_ref.clone(),
                art: a.article.clone(),
                s: a.sort.clone(),
                ok: a.adopte,
                mis: a.mission_visee.clone(),
                exp: a.expose_sommaire.clone(),
            }));
        }

        if !has_any {
            undated.push(json!({
                "id": a.id,
                "n": a.numero,
                "aid": a.auteur_id,
                "aty": a.auteur_type,
                "cos": a.cosignataires_ids,
                "did": a.dossier_ref,
                "art": a.article,
                "s": a.sort,
                "ok": a.adopte,
                "mis": a.mission_visee,
                "exp": a.expose_sommaire,
            }));
        }
    }

    // 2) Trier les évènements pour un rendu stable
    dated.sort_by(|(d1, o1, e1), (d2, o2, e2)| {
        d1.cmp(d2)
            .then(o1.cmp(o2))
            .then(e1.id.cmp(&e2.id))
    });

    // 3) Ecrire les shards mensuels (un mois à la fois)
    let mut month_stats: BTreeMap<String, MonthStat> = BTreeMap::new();

    let mut current_month: Option<String> = None;
    let mut days_map: BTreeMap<String, Vec<AmdEvent>> = BTreeMap::new();

    let mut flush_month = |month: &str, days: &BTreeMap<String, Vec<AmdEvent>>| -> Result<()> {
        let payload = AmendementsMonthFile {
            schema_version: 1,
            month: month.to_string(),
            days: days.clone(),
        };
        let path = months_dir.join(format!("{month}.json"));
        write_json_file(&path, &json!(payload))?;
        Ok(())
    };

    for (date, _order, ev) in dated.into_iter() {
        let month = date.format("%Y-%m").to_string();
        let day = date.to_string();

        let stat = month_stats.entry(month.clone()).or_insert_with(|| MonthStat {
            events: 0,
            days: HashSet::new(),
        });
        stat.events += 1;
        stat.days.insert(day.clone());

        if current_month.as_deref() != Some(&month) {
            if let Some(prev) = current_month.take() {
                flush_month(&prev, &days_map)?;
                days_map.clear();
            }
            current_month = Some(month.clone());
        }

        days_map.entry(day).or_default().push(ev);
    }
    if let Some(last) = current_month.take() {
        flush_month(&last, &days_map)?;
    }

    // 4) Index global
    let mut months: Vec<MonthMeta> = month_stats
        .into_iter()
        .map(|(month, stat)| MonthMeta {
            month,
            days: stat.days.len(),
            events: stat.events,
        })
        .collect();
    months.sort_by(|a, b| b.month.cmp(&a.month)); // plus récent d'abord

    let index = AmendementsIndex {
        schema_version: 1,
        generated_at: generated_at.to_string(),
        months,
        undated_count: undated.len(),
        undated_file: "data/amendements/undated.json".to_string(),
        notes: vec![
            "Chaque ligne est un évènement daté issu du cycle de vie open data (DEPOT / EXAMEN / SORT / CIRCULATION).".to_string(),
            "Certaines dates sont absentes dans l'open data : ces amendements sont comptés dans undated.json.".to_string(),
            "Les évènements sont shardés par mois dans data/amendements/months/YYYY-MM.json.".to_string(),
        ],
    };

    write_json_file(&amd_dir.join("index.json"), &json!(index))?;
    write_json_file(&amd_dir.join("undated.json"), &json!(undated))?;

    Ok(())
}

pub fn write_csv(agg: &AllAggregates, temp_dir: &Path) -> Result<()> {
    let exports_dir = temp_dir.join("exports");
    std::fs::create_dir_all(&exports_dir)?;

    write_period_csv(&exports_dir.join("deputes_activity_P30.csv"), &agg.p30)?;
    write_period_csv(&exports_dir.join("deputes_activity_P180.csv"), &agg.p180)?;
    write_period_csv(&exports_dir.join("deputes_activity_LEG.csv"), &agg.leg)?;

    Ok(())
}

fn write_period_csv(path: &Path, stats: &[DeputeStats]) -> Result<()> {
    let mut wtr = Writer::from_path(path)?;

    wtr.write_record(&[
        "deputy_id", "nom", "prenom",
        "groupe_abrev", "groupe_nom",
        "parti_rattachement",
        "dept", "circo",
        "period_start", "period_end",
        "scrutins_eligibles", "votes_exprimes", "non_votant", "absent", "participation_rate",
        "pour_count", "contre_count", "abst_count",
        "amd_authored", "amd_adopted", "amd_adoption_rate", "amd_cosigned",
        "interventions_count", "interventions_chars",
        "top_dossier_id", "top_dossier_titre", "top_dossier_score",
    ])?;

    for s in stats {
        let top = s.top_dossiers.first();
        wtr.write_record(&[
            &s.deputy_id,
            &s.nom,
            &s.prenom,
            s.groupe_abrev.as_deref().unwrap_or(""),
            s.groupe_nom.as_deref().unwrap_or(""),
            s.parti_rattachement.as_deref().unwrap_or(""),
            s.dept.as_deref().unwrap_or(""),
            s.circo.as_deref().unwrap_or(""),
            &s.period_start.to_string(),
            &s.period_end.to_string(),
            &s.scrutins_eligibles.to_string(),
            &s.votes_exprimes.to_string(),
            &s.non_votant.to_string(),
            &s.absent.to_string(),
            &format!("{:.4}", s.participation_rate),
            &s.pour_count.to_string(),
            &s.contre_count.to_string(),
            &s.abst_count.to_string(),
            &s.amd_authored.to_string(),
            &s.amd_adopted.to_string(),
            &s.amd_adoption_rate.map(|r| format!("{:.4}", r)).unwrap_or_default(),
            &s.amd_cosigned.to_string(),
            &s.interventions_count.to_string(),
            &s.interventions_chars.to_string(),
            top.map(|t| t.dossier_id.as_str()).unwrap_or(""),
            top.map(|t| t.titre.as_str()).unwrap_or(""),
            &top.map(|t| t.score.to_string()).unwrap_or_default(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

fn write_json_file(path: &Path, value: &serde_json::Value) -> Result<()> {
    let json = serde_json::to_string(value)?;
    let size_bytes = json.len();
    std::fs::write(path, json)?;
    eprintln!("[exporter] {} ({:.1} KB)", path.file_name().unwrap_or_default().to_string_lossy(), size_bytes as f64 / 1024.0);
    Ok(())
}
