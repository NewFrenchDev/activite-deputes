mod downloader;
mod models;
mod parser;
mod aggregator;
mod exporter;
mod group_ppl_v1;

use anyhow::Result;
use chrono::Utc;
use std::path::PathBuf;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("pipeline=debug,info"))
        )
        .init();

    info!("=== Démarrage pipeline activite-deputes ===");

    let work_dir = PathBuf::from("pipeline/.work");
    let temp_dir = PathBuf::from("pipeline/.temp_out");
    let site_data = PathBuf::from("site-dev/data");
    let site_exports = PathBuf::from("site-dev/exports");

    std::fs::create_dir_all(&work_dir)?;
    std::fs::create_dir_all(&temp_dir)?;
    std::fs::create_dir_all(&site_data)?;
    std::fs::create_dir_all(&site_exports)?;

    let sources = downloader::Sources::default();

    info!("Téléchargement des datasets...");
    let download_result = downloader::download_all(&sources, &work_dir).await;

    match download_result {
        Err(e) => {
            error!("Échec téléchargement critique: {e}");
            error!("Conservation de la dernière version publiée.");
            return Ok(());
        }
        Ok(etags) => {
            info!("Téléchargements OK");

            info!("Parsing des données...");
            let raw = match parser::parse_all(&work_dir) {
                Ok(r) => r,
                Err(e) => {
                    error!("Échec parsing: {e}");
                    return Ok(());
                }
            };
            info!("Parsing OK — {} députés, {} scrutins, {} amendements",
                raw.deputes.len(), raw.scrutins.len(), raw.amendements.len());

            info!("Calcul des agrégats...");
            let now = Utc::now();
            let aggregates = aggregator::compute_all(&raw, now)?;
            info!("Agrégats calculés");

            info!("Export JSON...");
            exporter::write_json(&aggregates, &temp_dir, &etags, now)?;

            info!("Export CSV...");
            exporter::write_csv(&aggregates, &temp_dir)?;

            info!("Swap atomique vers site/...");
            swap_output(&temp_dir, &site_data, &site_exports)?;

            info!("=== Pipeline terminé avec succès ===");
        }
    }

    Ok(())
}

fn swap_output(temp: &PathBuf, site_data: &PathBuf, site_exports: &PathBuf) -> Result<()> {
    let temp_data = temp.join("data");
    let temp_exports = temp.join("exports");

    if temp_data.exists() {
        if site_data.exists() {
            std::fs::remove_dir_all(site_data)?;
        }
        std::fs::create_dir_all(site_data.parent().unwrap())?;
        copy_dir_all(&temp_data, site_data)?;
    }

    if temp_exports.exists() {
        if site_exports.exists() {
            std::fs::remove_dir_all(site_exports)?;
        }
        std::fs::create_dir_all(site_exports.parent().unwrap())?;
        copy_dir_all(&temp_exports, site_exports)?;
    }

    std::fs::remove_dir_all(temp)?;
    Ok(())
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}
