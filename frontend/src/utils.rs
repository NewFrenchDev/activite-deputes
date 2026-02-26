pub fn fmt_pct(rate: f64) -> String {
    format!("{:.1}%", rate * 100.0)
}

pub fn participation_class(rate: f64) -> &'static str {
    if rate >= 0.75 {
        "participation-good"
    } else if rate >= 0.50 {
        "participation-mid"
    } else {
        "participation-low"
    }
}

pub fn groupe_color(abrev: Option<&str>) -> &'static str {
    match abrev {
        Some("RN") => "#3b82f6",
        Some("EPR") | Some("RE") => "#f59e0b",
        Some("LFI") => "#ef4444",
        Some("SOC") | Some("PS") => "#ec4899",
        Some("HOR") => "#06b6d4",
        Some("GDR") => "#dc2626",
        Some("Dem") | Some("MoDem") => "#a855f7",
        Some("LIOT") => "#84cc16",
        Some("UDR") => "#f97316",
        _ => "#6b7280",
    }
}

pub fn normalize_search(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            other => other,
        })
        .collect()
}

pub fn matches_search(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    normalize_search(haystack).contains(&normalize_search(needle))
}



fn app_base_path_from_base_tag() -> Option<String> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let base_el = document.query_selector("base[href]").ok().flatten()?;
    let href = base_el.get_attribute("href")?;
    let href = href.trim();
    if href.is_empty() {
        return None;
    }

    // Extrait un chemin utilisable pour le router à partir de <base href>.
    let path = if href.starts_with("http://") || href.starts_with("https://") {
        let after_scheme = href.splitn(2, "://").nth(1)?;
        let slash_pos = after_scheme.find('/').unwrap_or(after_scheme.len());
        let path_and_more = &after_scheme[slash_pos..];
        let path_only = path_and_more.split('#').next().unwrap_or(path_and_more);
        path_only.split('?').next().unwrap_or(path_only).to_string()
    } else if href.starts_with('/') {
        href.to_string()
    } else {
        format!("/{href}")
    };

    let trimmed = path.trim().trim_end_matches('/').to_string();
    if trimmed.is_empty() {
        Some("/".to_string())
    } else {
        Some(trimmed)
    }
}

/// Préfixe de l'application (sans slash final, sauf "/" pour la racine).
/// Source de vérité : la balise <base data-trunk-public-url /> injectée par Trunk.
pub fn app_base_path() -> String {
    if let Some(p) = app_base_path_from_base_tag() {
        return p;
    }

    // Fallback robuste si la balise <base> n'est pas disponible.
    let host = web_sys::window()
        .and_then(|w| w.location().hostname().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if host.ends_with(".github.io") {
        if let Some(pathname) = web_sys::window().and_then(|w| w.location().pathname().ok()) {
            let first = pathname.trim_start_matches('/').split('/').next().unwrap_or("").trim();
            if !first.is_empty() {
                return format!("/{first}");
            }
        }
    }

    "/".to_string()
}

/// Construit un href interne absolu compatible local + GitHub Pages.
pub fn app_href(path: &str) -> String {
    let p = path.trim();
    if p.is_empty() || p == "/" {
        let base = app_base_path();
        return if base == "/" { "/".to_string() } else { format!("{base}/") };
    }

    if p.starts_with("http://")
        || p.starts_with("https://")
        || p.starts_with("mailto:")
        || p.starts_with("tel:")
    {
        return p.to_string();
    }

    let base = app_base_path();
    let suffix = p.trim_start_matches('/');

    if base == "/" {
        format!("/{suffix}")
    } else {
        format!("{base}/{suffix}")
    }
}

pub fn is_local_dev_host() -> bool {
    let host = web_sys::window()
        .and_then(|w| w.location().hostname().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();

    matches!(host.as_str(), "localhost" | "127.0.0.1" | "0.0.0.0" | "[::1]")
}

#[macro_export]
macro_rules! app_path {
    ($suffix:literal) => {
        concat!("/activite-deputes", $suffix)
    };
}
