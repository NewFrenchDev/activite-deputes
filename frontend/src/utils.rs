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
