use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::json;
use std::path::Path;
use csv::Writer;
use tracing::info;

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

    // deputes.json — info de base pour le listing
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
    write_json_file(&data_dir.join("deputes.json"), &json!(deputes_base))?;

    // positions-groupes / PPL (V1) — shards par groupe pour limiter la bande passante
    group_ppl_v1::write_group_ppl_json(&agg.deputes, &agg.dossiers, &data_dir, &now.to_rfc3339())?;

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
    info!("{}: {} bytes", path.display(), size_bytes);
    Ok(())
}
