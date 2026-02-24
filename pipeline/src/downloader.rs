use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, warn};

pub struct Source {
    pub key: &'static str,
    pub url: &'static str,
    pub filename: &'static str,
}

pub struct Sources {
    pub list: Vec<Source>,
}

impl Default for Sources {
    fn default() -> Self {
        Self {
            list: vec![
                Source {
                    key: "deputes",
                    url: "http://data.assemblee-nationale.fr/static/openData/repository/17/amo/deputes_actifs_mandats_actifs_organes/AMO10_deputes_actifs_mandats_actifs_organes.json.zip",
                    filename: "deputes.zip",
                },
                Source {
                    key: "scrutins",
                    url: "http://data.assemblee-nationale.fr/static/openData/repository/17/loi/scrutins/Scrutins.json.zip",
                    filename: "scrutins.zip",
                },
                Source {
                    key: "amendements",
                    url: "http://data.assemblee-nationale.fr/static/openData/repository/17/loi/amendements_div_legis/Amendements.json.zip",
                    filename: "amendements.zip",
                },
                Source {
                    key: "dossiers",
                    url: "http://data.assemblee-nationale.fr/static/openData/repository/17/loi/dossiers_legislatifs/Dossiers_Legislatifs.json.zip",
                    filename: "dossiers.zip",
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct EtagInfo {
    pub key: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub size_bytes: u64,
}

pub async fn download_all(sources: &Sources, work_dir: &Path) -> Result<Vec<EtagInfo>> {
    let client = Arc::new(
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .user_agent("activite-deputes/1.0 (github.com; open-data-consumer)")
            .build()?
    );

    let etag_cache = Arc::new(load_etag_cache(work_dir));
    // Limite de 2 downloads simultanés pour ne pas surcharger le serveur AN
    let semaphore = Arc::new(Semaphore::new(2));
    let work_dir = Arc::new(work_dir.to_path_buf());

    let mut handles = Vec::new();

    for source in &sources.list {
        let client = Arc::clone(&client);
        let etag_cache = Arc::clone(&etag_cache);
        let sem = Arc::clone(&semaphore);
        let work_dir = Arc::clone(&work_dir);
        let key = source.key;
        let url = source.url;
        let filename = source.filename;

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            download_one(&client, &etag_cache, &work_dir, key, url, filename).await
        });
        handles.push((source.key, handle));
    }

    let mut results = Vec::new();
    let mut errors = Vec::new();

    for (key, handle) in handles {
        match handle.await {
            Ok(Ok(info)) => results.push(info),
            Ok(Err(e)) => {
                warn!("Téléchargement échoué pour {key}: {e}");
                errors.push(format!("{key}: {e}"));
            }
            Err(e) => {
                warn!("Task paniqué pour {key}: {e}");
                errors.push(format!("{key}: join error"));
            }
        }
    }

    if !errors.is_empty() {
        anyhow::bail!("Échecs de téléchargement: {}", errors.join(", "));
    }

    // Mettre à jour le cache ETag avec tous les résultats
    save_etag_cache(&work_dir, &results);

    Ok(results)
}

async fn download_one(
    client: &reqwest::Client,
    etag_cache: &HashMap<String, String>,
    work_dir: &Path,
    key: &'static str,
    url: &'static str,
    filename: &'static str,
) -> Result<EtagInfo> {
    info!("Téléchargement: {key} depuis {url}");
    let zip_path = work_dir.join(filename);
    let extract_dir = work_dir.join(key);

    let cached_etag = etag_cache.get(key).cloned();
    let mut req = client.get(url);
    if let Some(ref etag) = cached_etag {
        req = req.header("If-None-Match", etag);
    }

    let resp = req.send().await
        .with_context(|| format!("Requête HTTP {url}"))?;

    let status = resp.status();
    let etag = resp.headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let last_modified = resp.headers()
        .get("last-modified")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    // 304 Not Modified: rien à télécharger, mais on doit avoir le répertoire extrait
    if status == reqwest::StatusCode::NOT_MODIFIED && extract_dir.exists() {
        info!("{key}: inchangé (304), réutilisation du cache");
        return Ok(EtagInfo {
            key: key.to_string(),
            etag: cached_etag,
            last_modified,
            size_bytes: zip_path.metadata().map(|m| m.len()).unwrap_or(0),
        });
    }

    // Vérifier que la réponse est un succès
    if !status.is_success() {
        anyhow::bail!("HTTP {status} pour {url}");
    }

    let bytes = resp.bytes().await
        .with_context(|| format!("Lecture bytes {url}"))?;
    let size = bytes.len() as u64;

    // Écrire le ZIP
    tokio::fs::write(&zip_path, &bytes).await
        .with_context(|| format!("Écriture {}", zip_path.display()))?;

    info!("{key}: {size} octets téléchargés, décompression...");

    // Extraire dans un thread bloquant (zip n'est pas async)
    let zip_path_sync = zip_path.clone();
    let extract_dir_sync = extract_dir.clone();
    tokio::task::spawn_blocking(move || {
        extract_zip(&zip_path_sync, &extract_dir_sync)
    })
    .await
    .with_context(|| format!("Thread blocking {key}"))?
    .with_context(|| format!("Décompression {key}"))?;

    info!("{key}: extraction OK");

    Ok(EtagInfo {
        key: key.to_string(),
        etag,
        last_modified,
        size_bytes: size,
    })
}

fn extract_zip(zip_path: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut zf = archive.by_index(i)?;
        let name = zf.name().to_string();

        // Ignorer les répertoires et les chemins suspects (path traversal)
        if name.ends_with('/') || name.contains("..") {
            continue;
        }

        // Utiliser uniquement le nom de fichier, pas le chemin complet dans le ZIP
        let file_name = Path::new(&name)
            .file_name()
            .unwrap_or(std::ffi::OsStr::new(&name));
        let out_path = dest.join(file_name);

        let mut out = std::fs::File::create(&out_path)
            .with_context(|| format!("Création fichier {:?}", out_path))?;
        std::io::copy(&mut zf, &mut out)?;
    }
    Ok(())
}

fn load_etag_cache(work_dir: &Path) -> HashMap<String, String> {
    let cache_path = work_dir.join("etag_cache.json");
    std::fs::read_to_string(cache_path)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}

fn save_etag_cache(work_dir: &Path, etags: &[EtagInfo]) {
    let map: HashMap<&str, &str> = etags.iter()
        .filter_map(|e| e.etag.as_deref().map(|et| (e.key.as_str(), et)))
        .collect();
    if let Ok(json) = serde_json::to_string(&map) {
        let _ = std::fs::write(work_dir.join("etag_cache.json"), json);
    }
}
