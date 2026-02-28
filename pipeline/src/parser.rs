use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use tracing::{info, warn};

use crate::models::*;

// ─── OneOrMany: gère le pattern JSON de l'AN ({} quand 1 seul, [] quand plusieurs)
fn one_or_many(v: &serde_json::Value) -> Vec<serde_json::Value> {
    match v {
        serde_json::Value::Array(arr) => arr.clone(),
        serde_json::Value::Object(_) => vec![v.clone()],
        _ => vec![],
    }
}

fn opt_non_empty_str(v: &serde_json::Value) -> Option<String> {
    v.as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "null")
        .map(String::from)
}

fn opt_non_empty_textish(v: &serde_json::Value) -> Option<String> {
    opt_non_empty_str(v).or_else(|| opt_non_empty_str(&v["#text"]))
}

fn normalize_urlish(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return String::new();
    }
    if raw.starts_with("http://") || raw.starts_with("https://") {
        raw.to_string()
    } else {
        format!("https://{}", raw.trim_start_matches('/'))
    }
}

fn normalize_phoneish(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_profession_label(raw: &str) -> String {
    let s = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let s = s.trim();
    if let Some(rest) = s.strip_prefix('(') {
        if let Some((code, after_paren)) = rest.split_once(')') {
            if code.chars().all(|c| c.is_ascii_digit()) {
                let cleaned = after_paren
                    .trim_start()
                    .trim_start_matches('-')
                    .trim_start_matches('–')
                    .trim_start_matches('—')
                    .trim_start();
                if !cleaned.is_empty() {
                    return cleaned.to_string();
                }
            }
        }
    }
    s.to_string()
}

/// Longueur maximale (en caractères) de l'exposé sommaire exporté.
/// Au-delà, le texte est tronqué avec "…".
const EXPOSE_MAX_CHARS: usize = 500;

/// Nettoie un exposé sommaire brut issu de l'open data :
/// Decode numeric HTML entities: &#xHEX; and &#DEC; → Unicode character
fn decode_numeric_entities(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '&' && chars.peek() == Some(&'#') {
            let mut entity = String::from("&#");
            chars.next(); // consume '#'
            while let Some(&c) = chars.peek() {
                if c == ';' {
                    chars.next(); // consume ';'
                    break;
                }
                entity.push(c);
                chars.next();
                if entity.len() > 10 { break; } // safety limit
            }
            let body = &entity[2..];
            let code_point = if body.starts_with('x') || body.starts_with('X') {
                u32::from_str_radix(&body[1..], 16).ok()
            } else {
                body.parse::<u32>().ok()
            };
            match code_point.and_then(char::from_u32) {
                Some(decoded) => result.push(decoded),
                None => { result.push_str(&entity); result.push(';'); }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Normalise un exposé sommaire brut :
/// 1. Supprime les balises HTML (ex: <p>, <br>)
/// 2. Décode quelques entités HTML courantes (ex: &nbsp;, &amp;, &lt;, &gt;, &quot;, &#39;)
/// 3. Décode les entités numériques (ex: &#x00E9; → é, &#233; → é)
/// 4. Collapse les espaces multiples
/// 5. Tronque à `max_chars` caractères avec "…"
fn normalize_expose_sommaire(raw: &str, max_chars: usize) -> String {
    // Strip HTML tags
    let mut out = String::with_capacity(raw.len());
    let mut in_tag = false;
    // Track whether the last emitted character was whitespace to avoid
    // concatenating words when tags are adjacent to text without spaces.
    let mut last_was_space = true;
    for ch in raw.chars() {
        match ch {
            '<' => {
                // We are starting a tag after some text: ensure a separator.
                if !in_tag && !last_was_space {
                    out.push(' ');
                    last_was_space = true;
                }
                in_tag = true;
            }
            '>' => {
                in_tag = false;
            }
            _ if !in_tag => {
                out.push(ch);
                last_was_space = ch.is_whitespace();
            }
            _ => {}
        }
    }

    // Decode common HTML entities
    let out = out
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    // Decode numeric HTML entities: &#xHEX; and &#DEC;
    let out = decode_numeric_entities(&out);

    // Collapse whitespace
    let collapsed: String = out.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = collapsed.trim();

    if trimmed.is_empty() {
        return String::new();
    }

    // Truncate
    if max_chars == 0 || trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }

    let mut result = String::new();
    for (i, ch) in trimmed.chars().enumerate() {
        if i >= max_chars {
            result.push('…');
            break;
        }
        result.push(ch);
    }
    result
}

fn normalize_sexe_label(raw: &str) -> Option<String> {
    let s = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let t = s.trim();
    if t.is_empty() || t.eq_ignore_ascii_case("null") {
        return None;
    }
    let lower = t.to_lowercase();
    if matches!(lower.as_str(), "m" | "m." | "mr" | "monsieur" | "homme" | "masculin") {
        return Some("Homme".to_string());
    }
    if matches!(lower.as_str(), "mme" | "mme." | "madame" | "melle" | "mlle" | "femme" | "féminin" | "feminin") {
        return Some("Femme".to_string());
    }
    Some(t.to_string())
}

fn push_unique(vec: &mut Vec<String>, value: String) {
    if !value.is_empty() && !vec.iter().any(|v| v.eq_ignore_ascii_case(&value)) {
        vec.push(value);
    }
}

fn site_type_rank(type_libelle: &str, type_code: &str) -> i32 {
    let t = type_libelle.to_ascii_lowercase();
    if t.contains("site internet") || t.contains("site web") {
        return 0;
    }
    if t.contains("blog") {
        return 1;
    }
    if t.contains("réseau social") || t.contains("reseau social") {
        return 3;
    }
    match type_code {
        "22" => 0,
        _ => 2,
    }
}

fn is_type_organe_assemblee(raw: &str) -> bool {
    let t = raw.trim();
    matches!(t, "ASSEMBLEE" | "Assemblée" | "ASSEMBLÉE" | "Assemblee" | "assemblee" | "assemblée")
}

fn is_mandat_parlementaire_assemblee(m: &serde_json::Value) -> bool {
    let xsi_type = m["@xsi:type"].as_str().unwrap_or("");
    let type_organe = m["typeOrgane"].as_str().unwrap_or("");
    xsi_type == "MandatParlementaire_type" && is_type_organe_assemblee(type_organe)
}

pub fn parse_all(work_dir: &Path) -> Result<RawDataset> {
    let deputes_dir = work_dir.join("deputes");
    let scrutins_dir = work_dir.join("scrutins");
    let amendements_dir = work_dir.join("amendements");
    let dossiers_dir = work_dir.join("dossiers");

    let t_all = Instant::now();
    info!("Parsing détaillé: début");

    let t = Instant::now();
    let (deputes, organes) = parse_deputes(&deputes_dir)
        .context("Parsing députés")?;
    info!(
        "Parsing députés OK en {:?} (deputes={}, organes={})",
        t.elapsed(),
        deputes.len(),
        organes.len()
    );

    let t = Instant::now();
    let scrutins = parse_scrutins(&scrutins_dir)
        .context("Parsing scrutins")?;
    info!("Parsing scrutins OK en {:?} (scrutins={})", t.elapsed(), scrutins.len());

    let t = Instant::now();
    let amendements = parse_amendements(&amendements_dir)
        .context("Parsing amendements")?;
    info!(
        "Parsing amendements OK en {:?} (amendements={})",
        t.elapsed(),
        amendements.len()
    );

    let t = Instant::now();
    let dossiers = match parse_dossiers(&dossiers_dir) {
        Ok(d) => d,
        Err(e) => {
            warn!("Parsing dossiers échoué ({e}) — fallback dossiers vides");
            HashMap::new()
        }
    };
    info!("Parsing dossiers OK en {:?} (dossiers={})", t.elapsed(), dossiers.len());

    info!("Parsing détaillé: terminé en {:?}", t_all.elapsed());

    Ok(RawDataset { deputes, organes, scrutins, amendements, dossiers })
}

fn parse_deputes(dir: &Path) -> Result<(Vec<Depute>, HashMap<String, Organe>)> {
    if !dir.exists() {
        anyhow::bail!("Répertoire non trouvé: {:?}", dir);
    }

    let mut json_files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();

    json_files.sort();

    if json_files.is_empty() {
        anyhow::bail!("Aucun fichier JSON dans {:?}", dir);
    }

    // Heuristique: si on voit beaucoup de PA*/PO*, le ZIP est au format "multi-fichiers"
    // (1 fichier JSON par acteur / organe). L'ancien format est un JSON agrégé avec "export".
    let mut pa = 0usize;
    let mut po = 0usize;
    for p in &json_files {
        if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
            if name.starts_with("PA") {
                pa += 1;
            } else if name.starts_with("PO") {
                po += 1;
            }
        }
    }

    if pa + po >= 50 {
        info!("Députés: détection format multi-fichiers (acteurs≈{}, organes≈{})", pa, po);
        return parse_deputes_multifile(&json_files);
    }

    // Sinon, on tente le format agrégé (export.acteurs / export.organes)
    match parse_deputes_aggregated(&json_files) {
        Ok(ok) => Ok(ok),
        Err(e) => {
            warn!("Parsing format agrégé échoué ({e}), tentative multi-fichiers...");
            parse_deputes_multifile(&json_files)
        }
    }
}

fn parse_deputes_aggregated(json_files: &[std::path::PathBuf]) -> Result<(Vec<Depute>, HashMap<String, Organe>)> {
    let mut best_root: Option<serde_json::Value> = None;
    let mut best_score: usize = 0;
    let mut best_path: Option<std::path::PathBuf> = None;

    for path in json_files {
        let data = std::fs::read_to_string(path)
            .with_context(|| format!("Lecture {:?}", path))?;
        let root: serde_json::Value = serde_json::from_str(&data)
            .with_context(|| format!("JSON invalide {:?}", path))?;

        let export = root.get("export").unwrap_or(&root);
        let acteurs = one_or_many(&export["acteurs"]["acteur"]);
        let organes_raw = one_or_many(&export["organes"]["organe"]);

        let score = acteurs.len() + organes_raw.len();
        if score > best_score {
            best_score = score;
            best_root = Some(root);
            best_path = Some(path.clone());
        }
    }

    if best_score == 0 {
        anyhow::bail!("Aucun JSON agrégé exploitable trouvé (score=0)");
    }

    let root = best_root.unwrap();
    let picked = best_path.unwrap();

    let export = root.get("export").unwrap_or(&root);
    let acteurs = one_or_many(&export["acteurs"]["acteur"]);
    let organes_raw = one_or_many(&export["organes"]["organe"]);

    let organes: HashMap<String, Organe> = organes_raw.iter()
        .filter_map(parse_organe)
        .map(|o| (o.id.clone(), o))
        .collect();

    let deputes: Vec<Depute> = acteurs.iter()
        .filter_map(|a| parse_depute(a, &organes))
        .collect();

    if deputes.is_empty() {
        anyhow::bail!("0 député parsé depuis {:?} (format agrégé)", picked);
    }

    info!("Députés: format agrégé OK (deputes={}, organes={})", deputes.len(), organes.len());
    Ok((deputes, organes))
}

fn parse_deputes_multifile(json_files: &[std::path::PathBuf]) -> Result<(Vec<Depute>, HashMap<String, Organe>)> {
    let mut organes: HashMap<String, Organe> = HashMap::new();

    // Pass 1: organe (PO*.json)
    for path in json_files.iter().filter(|p| {
        p.file_name()
            .and_then(|s| s.to_str())
            .map(|n| n.starts_with("PO"))
            .unwrap_or(false)
    }) {
        let data = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                warn!("Organe: lecture échouée {:?}: {e}", path);
                continue;
            }
        };

        let root: serde_json::Value = match serde_json::from_str(&data) {
            Ok(v) => v,
            Err(e) => {
                warn!("Organe: JSON invalide {:?}: {e}", path);
                continue;
            }
        };

        let node = root.get("organe").unwrap_or(&root);
        if let Some(o) = parse_organe(node) {
            organes.insert(o.id.clone(), o);
        }
    }

    if organes.is_empty() {
        warn!("Députés: 0 organe parsé en mode multi-fichiers (les groupes/partis seront vides)");
    }

    // Pass 2: acteurs (PA*.json)
    let mut deputes: Vec<Depute> = Vec::new();
    for path in json_files.iter().filter(|p| {
        p.file_name()
            .and_then(|s| s.to_str())
            .map(|n| n.starts_with("PA"))
            .unwrap_or(false)
    }) {
        let data = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                warn!("Acteur: lecture échouée {:?}: {e}", path);
                continue;
            }
        };

        let root: serde_json::Value = match serde_json::from_str(&data) {
            Ok(v) => v,
            Err(e) => {
                warn!("Acteur: JSON invalide {:?}: {e}", path);
                continue;
            }
        };

        let node = root.get("acteur").unwrap_or(&root);
        if let Some(d) = parse_depute(node, &organes) {
            deputes.push(d);
        }
    }

    if deputes.is_empty() {
        anyhow::bail!(
            "0 député parsé en mode multi-fichiers. Vérifie la structure JSON (clé 'acteur')"
        );
    }

    info!("Députés: format multi-fichiers OK (deputes={}, organes={})", deputes.len(), organes.len());
    Ok((deputes, organes))
}

fn parse_organe(v: &serde_json::Value) -> Option<Organe> {
    // uid peut être une string directe ou un objet {"#text": "..."}
    let id = v["uid"]["#text"].as_str()
        .or_else(|| v["uid"].as_str())?
        .to_string();
    let code_type = v["codeType"].as_str()?.to_string();
    let libelle = v["libelle"].as_str().unwrap_or("").to_string();
    let abrev = v["libelleAbrev"].as_str().map(String::from);
    let couleur = v["couleurAssociee"].as_str().map(String::from);
    Some(Organe { id, code_type, libelle, abrev, couleur })
}

fn parse_depute(v: &serde_json::Value, organes: &HashMap<String, Organe>) -> Option<Depute> {
    let uid = v["uid"]["#text"].as_str()
        .or_else(|| v["uid"].as_str())?
        .to_string();

    let etat_civil = &v["etatCivil"]["ident"];
    let nom = etat_civil["nom"].as_str().unwrap_or("").to_string();
    let prenom = etat_civil["prenom"].as_str().unwrap_or("").to_string();

    let date_naissance = v.pointer("/etatCivil/infoNaissance/dateNais")
        .and_then(|x| x.as_str())
        .and_then(parse_date)
        .or_else(|| {
            v.pointer("/etatCivil/infoNaissKnown/dateNais")
                .and_then(|x| x.as_str())
                .and_then(parse_date)
        });

    let sexe = v.pointer("/etatCivil/ident/sexe")
        .and_then(opt_non_empty_textish)
        .and_then(|s| normalize_sexe_label(&s))
        .or_else(|| {
            v.pointer("/etatCivil/ident/civ")
                .and_then(opt_non_empty_textish)
                .and_then(|s| normalize_sexe_label(&s))
        });

    let pays_naissance = v.pointer("/etatCivil/infoNaissance/paysNais")
        .and_then(opt_non_empty_textish)
        .or_else(|| {
            v.pointer("/etatCivil/infoNaissKnown/paysNais")
                .and_then(opt_non_empty_textish)
        });

    let profession = v.pointer("/profession/libelleCourant")
        .and_then(|x| x.as_str())
        .map(normalize_profession_label)
        .or_else(|| {
            v.pointer("/professions/profession")
                .and_then(|x| x.as_str())
                .map(normalize_profession_label)
        })
        .filter(|s| !s.trim().is_empty());

    let uri_hatvp = opt_non_empty_str(&v["uri_hatvp"])
        .map(|u| normalize_urlish(&u))
        .filter(|u| !u.is_empty());

    let mut email_assemblee: Option<String> = None;
    let mut telephones: Vec<String> = Vec::new();
    let mut site_candidates: Vec<(i32, String)> = Vec::new();
    let mut site_sources_candidates: Vec<(i32, SiteWebSource)> = Vec::new();

    for adr in one_or_many(&v["adresses"]["adresse"]) {
        let adr_type = adr["@xsi:type"].as_str().unwrap_or("");
        let adr_code = adr["type"].as_str().unwrap_or("");
        let type_libelle = adr["typeLibelle"].as_str().unwrap_or("");
        let raw_val = opt_non_empty_str(&adr["valElec"])
            .or_else(|| opt_non_empty_str(&adr["valeur"]));

        match adr_type {
            "AdresseMail_Type" => {
                if email_assemblee.is_none() {
                    email_assemblee = raw_val.clone();
                }
            }
            "AdresseSiteWeb_Type" => {
                if let Some(raw_site) = raw_val.clone() {
                    let rank = site_type_rank(type_libelle, adr_code);
                    let normalized_url = normalize_urlish(&raw_site);
                    if !normalized_url.is_empty() {
                        site_candidates.push((rank, normalized_url.clone()));
                    }
                    let type_label = opt_non_empty_str(&adr["typeLibelle"]);
                    site_sources_candidates.push((
                        rank,
                        SiteWebSource {
                            type_libelle: type_label,
                            val_elec: raw_site,
                            url: if normalized_url.is_empty() {
                                None
                            } else {
                                Some(normalized_url)
                            },
                        },
                    ));
                }
            }
            "AdresseTelephonique_Type" => {
                if let Some(tel) = raw_val
                    .as_deref()
                    .map(normalize_phoneish)
                    .filter(|t| !t.is_empty())
                {
                    push_unique(&mut telephones, tel);
                }
            }
            _ => {}
        }
    }

    site_candidates.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    let mut sites_web: Vec<String> = Vec::new();
    for (_rank, url) in site_candidates {
        push_unique(&mut sites_web, url);
    }
    let mut sites_web_sources: Vec<SiteWebSource> = Vec::new();
    site_sources_candidates.sort_by(|a, b| a.0.cmp(&b.0));
    for (_rank, entry) in site_sources_candidates {
        if !sites_web_sources.iter().any(|e| e == &entry) {
            sites_web_sources.push(entry);
        }
    }
    let site_web = sites_web.first().cloned();

    // Utiliser one_or_many pour gérer tous les cas de la structure mandats
    let mandats = one_or_many(&v["mandats"]["mandat"]);

    let mut dept_code: Option<String> = None;
    let mut dept_nom: Option<String> = None;
    let mut circo: Option<String> = None;
    let mut mandat_debut: Option<NaiveDate> = None;
    let mut mandat_fin: Option<NaiveDate> = None;
    let mut mandat_match_level: i8 = 0; // 2 = strict (MandatParlementaire_type + Assemblée), 1 = fallback legacy
    let mut mandat_is_active: bool = false;

    #[derive(Clone)]
    struct MandatCandidateWindow {
        debut: Option<NaiveDate>,
        fin: Option<NaiveDate>,
    }
    let mut mandat_windows_level: i8 = 0;
    let mut mandat_windows_candidates: Vec<MandatCandidateWindow> = Vec::new();

    // Pour les groupes : on veut le GP actif le plus récent (dateDebut max sans dateFin)
    // Si plusieurs GP trouvés, on prend celui dont dateDebut est la plus récente
    let mut groupe_id: Option<String> = None;
    let mut groupe_abrev: Option<String> = None;
    let mut groupe_nom: Option<String> = None;
    let mut groupe_debut: Option<NaiveDate> = None;

    let mut parti_id: Option<String> = None;
    let mut parti_nom: Option<String> = None;
    let mut parti_debut: Option<NaiveDate> = None;

    for m in &mandats {
        let type_organe = m["typeOrgane"].as_str().unwrap_or("");

        // Mandat député (17e législature) : privilégier le mandat parlementaire à l'Assemblée
        // (certaines fiches contiennent d'autres mandats liés à l'Assemblée qui faussent la date de début).
        let strict_mandat_match = is_mandat_parlementaire_assemblee(m);
        let legacy_mandat_match = is_type_organe_assemblee(type_organe);
        let mandat_level: i8 = if strict_mandat_match {
            2
        } else if legacy_mandat_match {
            1
        } else {
            0
        };

        if mandat_level > 0 {
            let debut = m["dateDebut"].as_str().and_then(parse_date);
            let fin = m["dateFin"].as_str().and_then(parse_date);

            if mandat_level > mandat_windows_level {
                mandat_windows_level = mandat_level;
                mandat_windows_candidates.clear();
            }
            if mandat_level == mandat_windows_level {
                mandat_windows_candidates.push(MandatCandidateWindow { debut, fin });
            }

            let candidate_active = fin.is_none();
            let should_take = mandat_level > mandat_match_level
                || (mandat_level == mandat_match_level && candidate_active && !mandat_is_active)
                || (mandat_level == mandat_match_level
                    && candidate_active == mandat_is_active
                    && debut > mandat_debut);

            if should_take {
                let circ = &m["election"]["lieu"];
                dept_code = circ["numDepartement"].as_str()
                    .or_else(|| circ["numDpt"].as_str())
                    .map(String::from)
                    .or(dept_code.clone());
                dept_nom = circ["departement"].as_str()
                    .or_else(|| circ["nomDpt"].as_str())
                    .map(String::from)
                    .or(dept_nom.clone());
                circo = circ["numCirco"].as_str().map(String::from).or(circo.clone());

                mandat_debut = debut;
                mandat_fin = fin;
                mandat_match_level = mandat_level;
                mandat_is_active = candidate_active;
            }
        }

        // Groupe parlementaire — on retient le plus récent par dateDebut
        if type_organe == "GP" {
            let debut = m["dateDebut"].as_str().and_then(parse_date);
            let fin = m["dateFin"].as_str().and_then(parse_date);
            // Ignorer les mandats de groupe terminés (sauf si c'est le seul)
            let is_active = fin.is_none();
            let is_more_recent = debut > groupe_debut;

            if is_active || groupe_id.is_none() || is_more_recent {
                // Résoudre organeRef (peut être string ou objet)
                let org_ref = m["organes"]["organeRef"].as_str()
                    .map(String::from);
                if let Some(ref org_ref_str) = org_ref {
                    if let Some(org) = organes.get(org_ref_str) {
                        // On met à jour seulement si actif OU aucun groupe encore trouvé
                        if is_active || groupe_id.is_none() || is_more_recent {
                            groupe_id = Some(org_ref_str.clone());
                            groupe_abrev = org.abrev.clone();
                            groupe_nom = Some(org.libelle.clone());
                            groupe_debut = debut;
                        }
                    }
                }
            }
        }

        // Parti politique — même logique: le plus récent actif
        if type_organe == "PARPOL" {
            let debut = m["dateDebut"].as_str().and_then(parse_date);
            let fin = m["dateFin"].as_str().and_then(parse_date);
            let is_active = fin.is_none();
            let is_more_recent = debut > parti_debut;

            if is_active || parti_id.is_none() || is_more_recent {
                let org_ref = m["organes"]["organeRef"].as_str().map(String::from);
                if let Some(ref org_ref_str) = org_ref {
                    parti_id = Some(org_ref_str.clone());
                    parti_debut = debut;
                    if let Some(org) = organes.get(org_ref_str) {
                        parti_nom = Some(org.libelle.clone());
                    }
                }
            }
        }
    }

    let mut mandat_assemblee_episodes: Vec<MandatAssembleeEpisode> = mandat_windows_candidates
        .into_iter()
        .filter_map(|w| {
            w.debut.map(|date_debut| MandatAssembleeEpisode {
                date_debut,
                date_fin: w.fin,
            })
        })
        .collect();
    mandat_assemblee_episodes.sort_by(|a, b| {
        a.date_debut
            .cmp(&b.date_debut)
            .then_with(|| a.date_fin.cmp(&b.date_fin))
    });
    mandat_assemblee_episodes.dedup_by(|a, b| a.date_debut == b.date_debut && a.date_fin == b.date_fin);

    let mandat_debut_legislature = mandat_assemblee_episodes
        .first()
        .map(|e| e.date_debut)
        .or(mandat_debut);

    Some(Depute {
        id: uid,
        nom,
        prenom,
        date_naissance,
        sexe,
        pays_naissance,
        profession,
        dept_code,
        dept_nom,
        circo,
        mandat_debut,
        mandat_fin,
        mandat_debut_legislature,
        mandat_assemblee_episodes,
        groupe_id,
        groupe_abrev,
        groupe_nom,
        parti_id,
        parti_nom,
        email_assemblee,
        site_web,
        sites_web,
        sites_web_sources,
        telephones,
        uri_hatvp,
    })
}

fn parse_scrutins(dir: &Path) -> Result<Vec<Scrutin>> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut scrutins = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            match parse_scrutin_file(&path) {
                Ok(mut v) => scrutins.append(&mut v),
                Err(e) => warn!("Scrutin file {:?}: {e}", path),
            }
        }
    }

    let total_votes: usize = scrutins.iter().map(|s| s.votes.len()).sum();
    let avg = if scrutins.is_empty() {
        0.0
    } else {
        total_votes as f64 / scrutins.len() as f64
    };
    info!(
        "Scrutins: {} scrutins, {} votes nominaux extraits (moyenne {:.1}/scrutin)",
        scrutins.len(),
        total_votes,
        avg
    );

    Ok(scrutins)
}

fn parse_scrutin_file(path: &Path) -> Result<Vec<Scrutin>> {
    let data = std::fs::read_to_string(path)?;
    let root: serde_json::Value = serde_json::from_str(&data)?;

    // Plusieurs structures possibles selon le fichier AN
    let scrutins_raw = one_or_many(&root["scrutins"]["scrutin"])
        .into_iter()
        .chain(one_or_many(&root["scrutin"]))
        .collect::<Vec<_>>();

    // Dédupliquer par uid pour éviter les doublons si les deux clés existent
    let mut seen = std::collections::HashSet::new();
    let scrutins_raw: Vec<_> = scrutins_raw.into_iter()
        .filter(|s| {
            if let Some(uid) = s["uid"].as_str() {
                seen.insert(uid.to_string())
            } else {
                true
            }
        })
        .collect();

    Ok(scrutins_raw.iter().filter_map(parse_scrutin).collect())
}

fn parse_scrutin(v: &serde_json::Value) -> Option<Scrutin> {
    let id = v["uid"].as_str()?.to_string();
    let numero = v["numero"].as_str()
        .and_then(|s| s.parse::<u32>().ok())
        .or_else(|| v["numero"].as_u64().map(|n| n as u32))
        .unwrap_or(0);
    let titre = v["titre"].as_str()
        .or_else(|| v["objet"]["libelle"].as_str())
        .unwrap_or("")
        .to_string();
    let date = v["dateScrutin"].as_str().and_then(parse_date);
    let sort = v["sort"]["value"].as_str()
        .or_else(|| v["sort"]["libelle"].as_str())
        .or_else(|| v["sort"]["code"].as_str())
        .or_else(|| v["sort"].as_str())
        .map(String::from);
    let dossier_ref = v["dossierRef"].as_str()
        .or_else(|| v["objet"]["dossierLegislatif"].as_str())
        .map(String::from);

    let mut votes: HashMap<String, VotePosition> = HashMap::new();

    // Schéma AN observé (scrutins.zip):
    // ventilationVotes.organe.groupes.groupe[].vote.decompteNominatif.{pours,contres,abstentions,nonVotants}.votant
    // + fallbacks pour variantes historiques/agrégées.
    let org_nodes = one_or_many(&v["ventilationVotes"]["organe"]);

    for org in &org_nodes {
        let groupes = one_or_many(&org["groupes"]["groupe"])
            .into_iter()
            .chain(one_or_many(&org["groupes"]["organe"]))
            .chain(one_or_many(&org["groupe"]))
            .chain(one_or_many(&org["organe"]))
            .collect::<Vec<_>>();

        for groupe in &groupes {
            let decompte = if groupe["vote"]["decompteNominatif"].is_object() {
                &groupe["vote"]["decompteNominatif"]
            } else if groupe["votes"].is_object() {
                &groupe["votes"]
            } else if groupe["vote"].is_object() {
                &groupe["vote"]
            } else {
                continue;
            };

            let cats = [
                ("pours", VotePosition::Pour),
                ("pour", VotePosition::Pour),
                ("contres", VotePosition::Contre),
                ("contre", VotePosition::Contre),
                ("abstentions", VotePosition::Abstention),
                ("abstention", VotePosition::Abstention),
                ("nonVotants", VotePosition::NonVotant),
                ("nonVotant", VotePosition::NonVotant),
            ];

            for (cat, pos) in &cats {
                let bucket = &decompte[*cat];
                if bucket.is_null() {
                    continue;
                }

                let votants = one_or_many(&bucket["votant"])
                    .into_iter()
                    .chain(one_or_many(&bucket["votants"]["votant"]))
                    .collect::<Vec<_>>();

                for votant in votants {
                    let dep_id = votant["acteurRef"].as_str()
                        .or_else(|| votant["acteur"]["acteurRef"].as_str())
                        .or_else(|| votant["acteur"]["uid"].as_str())
                        .or_else(|| votant["uid"].as_str());
                    if let Some(dep_id) = dep_id {
                        votes.insert(dep_id.to_string(), pos.clone());
                    }
                }
            }
        }
    }

    Some(Scrutin { id, numero, titre, date, sort, dossier_ref, votes })
}

fn parse_amendements(dir: &Path) -> Result<Vec<Amendement>> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut amendements = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            match parse_amendement_file(&path) {
                Ok(mut v) => amendements.append(&mut v),
                Err(e) => warn!("Amendement file {:?}: {e}", path),
            }
        }
    }
    Ok(amendements)
}

fn parse_amendement_file(path: &Path) -> Result<Vec<Amendement>> {
    let data = std::fs::read_to_string(path)?;
    let root: serde_json::Value = serde_json::from_str(&data)?;

    let arr = one_or_many(&root["amendements"]["amendement"])
        .into_iter()
        .chain(one_or_many(&root["amendement"]))
        .collect::<Vec<_>>();

    if arr.is_empty() {
        return Ok(vec![]);
    }

    // Dédupliquer par uid
    let mut seen = std::collections::HashSet::new();
    let arr: Vec<_> = arr.into_iter()
        .filter(|a| {
            if let Some(uid) = a["uid"].as_str() {
                seen.insert(uid.to_string())
            } else {
                true
            }
        })
        .collect();

    Ok(arr.iter().filter_map(parse_amendement).collect())
}

fn parse_amendement(v: &serde_json::Value) -> Option<Amendement> {
    let id = v["uid"].as_str()?.to_string();
    let numero = v["identificatif"]["numero"].as_str().map(String::from);

    // Chaîne de fallback pour le sort : plusieurs emplacements possibles dans le JSON AN
    // IMPORTANT: dans les exports multi-fichiers 17e législature, `cycleDeVie.sort` est souvent
    // une chaîne directe (ex: "Adopté", "Rejeté"), pas un objet `{ value: ... }`.
    // Si on lit `etat.libelle` avant, on récupère souvent "Discuté" => adopte=false partout.
    let sort_val = v["cycleDeVie"]["sort"].as_str().map(String::from)
        .or_else(|| v["cycleDeVie"]["sort"]["value"].as_str().map(String::from))
        .or_else(|| v["cycleDeVie"]["etatDesTraitements"]["sousEtat"]["libelle"].as_str().map(String::from))
        .or_else(|| v["cycleDeVie"]["etatDesTraitements"]["sousEtat"]["code"].as_str().map(String::from))
        .or_else(|| v["cycleDeVie"]["etatDesTraitements"]["etat"]["libelle"].as_str().map(String::from))
        .or_else(|| v["cycleDeVie"]["etatDesTraitements"]["etat"]["code"].as_str().map(String::from));

    let adopte = sort_val.as_deref()
        .map(|s| {
            let s = s.to_lowercase();
            s.contains("adopt") || s == "29" // code 29 = adopté dans certains datasets
        })
        .unwrap_or(false);

    // auteur principal — peut être sous plusieurs clés
    let auteur_id = v["signataires"]["auteur"]["acteurRef"].as_str()
        .or_else(|| v["signataires"]["signataire"]["acteurRef"].as_str())
        .map(String::from);

    // typeAuteur (ex: "Député", "Groupe")
    let auteur_type = v["signataires"]["auteur"]["typeAuteur"].as_str()
        .or_else(|| v["signataires"]["signataire"]["typeAuteur"].as_str())
        .map(String::from);

    // cosignataires (schéma 17e législature observé):
    // signataires.cosignataires.acteurRef = "PA..." | ["PA...", ...]
    // + fallback historique: signataires.cosignataires.cosignataire[].acteurRef
    let mut cosignataires_ids: Vec<String> = Vec::new();
    let cosignataires = &v["signataires"]["cosignataires"];

    match &cosignataires["acteurRef"] {
        serde_json::Value::String(s) => {
            let s = s.trim();
            if !s.is_empty() {
                cosignataires_ids.push(s.to_string());
            }
        }
        serde_json::Value::Array(arr) => {
            for id in arr.iter().filter_map(|x| x.as_str()) {
                let id = id.trim();
                if !id.is_empty() {
                    cosignataires_ids.push(id.to_string());
                }
            }
        }
        _ => {}
    }

    if cosignataires_ids.is_empty() {
        cosignataires_ids = one_or_many(&cosignataires["cosignataire"])
            .into_iter()
            .filter_map(|c| c["acteurRef"].as_str().map(String::from))
            .collect();
    }

    cosignataires_ids.sort();
    cosignataires_ids.dedup();

    // Dates structurées (si présentes)
    let date_depot = v["cycleDeVie"]["dateDepot"].as_str().and_then(parse_date);
    let date_circulation = v["cycleDeVie"]["dateCirculation"].as_str().and_then(parse_date);
    let date_sort = v["cycleDeVie"]["dateSort"].as_str().and_then(parse_date);
    let date_examen = v["cycleDeVie"]["dateExamen"].as_str().and_then(parse_date);

    // Date best-effort : chaîne de fallback étendue — le dataset AN est très irrégulier sur ce point
    let date = date_depot
        .or(date_circulation)
        .or(date_sort)
        .or(date_examen)
        .or_else(|| {
            // Fallback: chercher n'importe quelle date dans cycleDeVie
            if let Some(obj) = v["cycleDeVie"].as_object() {
                for (_, val) in obj {
                    if let Some(s) = val.as_str() {
                        if let Some(d) = parse_date(s) {
                            return Some(d);
                        }
                    }
                }
            }
            None
        });

    let dossier_ref = v["dossierRef"].as_str().map(String::from)
        .or_else(|| v["pointeurFragmentTexte"]["texteLegislatifRef"].as_str().map(String::from));
    let article = v["pointeurFragmentTexte"]["division"]["titre"].as_str().map(String::from);
    let texte_ref = v["pointeurFragmentTexte"]["texteLegislatifRef"].as_str().map(String::from);

    // Mission visée
    let mission_visee = v["pointeurFragmentTexte"]["missionVisee"]["libelleMission"].as_str().map(String::from);
    let mission_ref = v["pointeurFragmentTexte"]["missionVisee"]["missionRef"].as_str().map(String::from);

    // Exposé sommaire — nettoyé (HTML strippé, whitespace collapsé, longueur limitée)
    let expose_sommaire = v["exposeSommaire"].as_str()
        .or_else(|| v["corps"]["contenuAuteur"]["exposeSommaire"].as_str())
        .map(|s| normalize_expose_sommaire(s, EXPOSE_MAX_CHARS))
        .filter(|s| !s.is_empty());

    Some(Amendement {
        id,
        numero,
        auteur_id,
        auteur_type,
        cosignataires_ids,
        sort: sort_val,
        date,
        date_depot,
        date_circulation,
        date_examen,
        date_sort,
        dossier_ref,
        article,
        texte_ref,
        adopte,
        mission_visee,
        mission_ref,
        expose_sommaire,
    })
}

fn parse_dossiers(dir: &Path) -> Result<HashMap<String, Dossier>> {
    if !dir.exists() {
        return Ok(HashMap::new());
    }
    let mut dossiers = HashMap::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let data = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                warn!("Dossier: lecture échouée {:?}: {e}", path);
                continue;
            }
        };

        let root: serde_json::Value = match serde_json::from_str(&data) {
            Ok(v) => v,
            Err(e) => {
                warn!("Dossier: JSON invalide {:?}: {e}", path);
                continue;
            }
        };

        // Supporte:
        // 1) format agrégé: {"dossiers":{"dossier":[...]}}
        // 2) format multi-fichier AN: {"dossierParlementaire":{...}}
        // 3) fallback: objet dossier directement en racine
        let mut inserted_any = false;

        for d in one_or_many(&root["dossiers"]["dossier"]) {
            if let Some(dossier) = parse_dossier(&d) {
                dossiers.insert(dossier.id.clone(), dossier);
                inserted_any = true;
            }
        }

        if !inserted_any {
            let single = root.get("dossierParlementaire").unwrap_or(&root);
            if let Some(dossier) = parse_dossier(single) {
                dossiers.insert(dossier.id.clone(), dossier);
            }
        }
    }
    Ok(dossiers)
}

fn collect_actor_refs_from_value(v: &serde_json::Value, out: &mut Vec<String>) {
    match v {
        serde_json::Value::Object(map) => {
            for (k, val) in map {
                if k == "acteurRef" {
                    match val {
                        serde_json::Value::String(s) => {
                            let s = s.trim();
                            if !s.is_empty() {
                                out.push(s.to_string());
                            }
                        }
                        serde_json::Value::Array(arr) => {
                            for item in arr {
                                if let Some(s) = item.as_str() {
                                    let s = s.trim();
                                    if !s.is_empty() {
                                        out.push(s.to_string());
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    collect_actor_refs_from_value(val, out);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                collect_actor_refs_from_value(item, out);
            }
        }
        _ => {}
    }
}

fn dedup_actor_refs_keep_order(ids: &mut Vec<String>) {
    let mut seen = std::collections::HashSet::<String>::new();
    ids.retain(|id| {
        let key = id.trim().to_string();
        if key.is_empty() {
            return false;
        }
        seen.insert(key)
    });
}

fn parse_dossier_signers(v: &serde_json::Value) -> (Option<String>, Vec<String>) {
    // Pattern prioritaire (analogue aux amendements)
    let mut auteur_id = v["signataires"]["auteur"]["acteurRef"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let mut cosignataires_ids: Vec<String> = Vec::new();
    let cosignataires = &v["signataires"]["cosignataires"];
    match &cosignataires["acteurRef"] {
        serde_json::Value::String(s) => {
            let s = s.trim();
            if !s.is_empty() {
                cosignataires_ids.push(s.to_string());
            }
        }
        serde_json::Value::Array(arr) => {
            for id in arr.iter().filter_map(|x| x.as_str()) {
                let id = id.trim();
                if !id.is_empty() {
                    cosignataires_ids.push(id.to_string());
                }
            }
        }
        _ => {}
    }

    if cosignataires_ids.is_empty() {
        cosignataires_ids = one_or_many(&cosignataires["cosignataire"])
            .into_iter()
            .filter_map(|c| c["acteurRef"].as_str().map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Fallback 1: bloc auteurs[] / auteur[]
    if auteur_id.is_none() {
        let mut refs = Vec::new();
        collect_actor_refs_from_value(&v["auteurs"], &mut refs);
        dedup_actor_refs_keep_order(&mut refs);
        if let Some(first) = refs.first().cloned() {
            auteur_id = Some(first.clone());
            for id in refs.into_iter().skip(1) {
                cosignataires_ids.push(id);
            }
        }
    }

    // Fallback 2: bloc signataires générique
    if auteur_id.is_none() {
        let mut refs = Vec::new();
        collect_actor_refs_from_value(&v["signataires"], &mut refs);
        dedup_actor_refs_keep_order(&mut refs);
        if let Some(first) = refs.first().cloned() {
            auteur_id = Some(first.clone());
            for id in refs.into_iter().skip(1) {
                cosignataires_ids.push(id);
            }
        }
    }

    // Fallback 3: dossiers parlementaires AN -> `initiateur.acteurs.acteur`
    // Peut être un objet (auteur seul) ou une liste (auteur + co-signataires).
    if auteur_id.is_none() {
        let mut refs = Vec::new();
        collect_actor_refs_from_value(&v["initiateur"], &mut refs);
        dedup_actor_refs_keep_order(&mut refs);
        if let Some(first) = refs.first().cloned() {
            auteur_id = Some(first.clone());
            for id in refs.into_iter().skip(1) {
                cosignataires_ids.push(id);
            }
        }
    }

    dedup_actor_refs_keep_order(&mut cosignataires_ids);

    if let Some(aid) = &auteur_id {
        cosignataires_ids.retain(|id| id != aid);
    }

    (auteur_id, cosignataires_ids)
}

fn first_organe_ref_uid(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::Object(map) => {
            for (k, val) in map {
                if k == "organeRef" {
                    match val {
                        serde_json::Value::String(s) => {
                            let s = s.trim();
                            if !s.is_empty() {
                                return Some(s.to_string());
                            }
                        }
                        serde_json::Value::Object(obj) => {
                            if let Some(uid) = obj.get("uid").and_then(|x| x.as_str()) {
                                let uid = uid.trim();
                                if !uid.is_empty() {
                                    return Some(uid.to_string());
                                }
                            }
                        }
                        serde_json::Value::Array(arr) => {
                            for item in arr {
                                if let Some(uid) = first_organe_ref_uid(item) {
                                    return Some(uid);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if let Some(uid) = first_organe_ref_uid(val) {
                    return Some(uid);
                }
            }
            None
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                if let Some(uid) = first_organe_ref_uid(item) {
                    return Some(uid);
                }
            }
            None
        }
        _ => None,
    }
}

fn detect_dossier_origin(v: &serde_json::Value) -> (Option<String>, Option<String>) {
    let initiateur_organe_ref = first_organe_ref_uid(&v["initiateur"]);

    let senat_chemin = v["titreDossier"]["senatChemin"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let origin_chamber = if senat_chemin.is_some()
        || initiateur_organe_ref.as_deref() == Some("PO78718")
    {
        Some("senat".to_string())
    } else {
        Some("assemblee".to_string())
    };

    (origin_chamber, initiateur_organe_ref)
}

fn parse_dossier(v: &serde_json::Value) -> Option<Dossier> {
    let id = v["uid"].as_str()?.to_string();
    let titre = v["titreDossier"]["titre"].as_str()
        .filter(|s| !s.is_empty())
        .or_else(|| v["titre"].as_str())
        .unwrap_or("")
        .to_string();
    let date_depot = v["titreDossier"]["dateDepot"].as_str().and_then(parse_date)
        .or_else(|| v["dateDepot"].as_str().and_then(parse_date));
    let statut = v["procedureParlementaire"]["libelle"].as_str().map(String::from);
    let legislature = v["legislature"].as_str().map(String::from);
    let nature = v["nature"].as_str().map(String::from)
        .or_else(|| v["titreDossier"]["titreChemin"].as_str().map(String::from));
    let numero = v["numero"].as_str().map(String::from)
        .or_else(|| v["titreDossier"]["numero"].as_str().map(String::from))
        .or_else(|| v["reference"]["numero"].as_str().map(String::from));
    let source_url = v["urlDossier"].as_str().map(String::from)
        .or_else(|| v["liens"]["lien"]["url"].as_str().map(String::from));

    let (origin_chamber, initiateur_organe_ref) = detect_dossier_origin(v);
    let (auteur_id, cosignataires_ids) = parse_dossier_signers(v);

    Some(Dossier {
        id,
        titre,
        date_depot,
        statut,
        legislature,
        nature,
        numero,
        auteur_id,
        cosignataires_ids,
        source_url,
        origin_chamber,
        initiateur_organe_ref,
    })
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if s.is_empty() || s == "null" {
        return None;
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
        .or_else(|| NaiveDate::parse_from_str(s, "%d/%m/%Y").ok())
        .or_else(|| {
            // Truncate datetime strings (ex: "2023-01-15T00:00:00")
            if s.len() >= 10 {
                NaiveDate::parse_from_str(&s[..10], "%Y-%m-%d").ok()
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── parse_date ────────────────────────────────────────────────────────
    #[test]
    fn parse_date_iso() {
        assert_eq!(parse_date("2023-06-19"), Some(NaiveDate::from_ymd_opt(2023, 6, 19).unwrap()));
    }

    #[test]
    fn parse_date_french() {
        assert_eq!(parse_date("19/06/2023"), Some(NaiveDate::from_ymd_opt(2023, 6, 19).unwrap()));
    }

    #[test]
    fn parse_date_datetime() {
        assert_eq!(parse_date("2023-01-15T00:00:00"), Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()));
    }

    #[test]
    fn parse_date_empty_or_null() {
        assert_eq!(parse_date(""), None);
        assert_eq!(parse_date("null"), None);
        assert_eq!(parse_date("  "), None);
    }

    #[test]
    fn parse_date_garbage() {
        assert_eq!(parse_date("not-a-date"), None);
    }

    // ─── normalize_expose_sommaire ─────────────────────────────────────────
    #[test]
    fn expose_strips_html_tags() {
        let raw = "<p>Cet amendement <b>vise</b> à clarifier.</p>";
        let result = normalize_expose_sommaire(raw, 500);
        assert_eq!(result, "Cet amendement vise à clarifier.");
    }

    #[test]
    fn expose_collapses_whitespace() {
        let raw = "  Texte   avec   espaces   multiples  ";
        let result = normalize_expose_sommaire(raw, 500);
        assert_eq!(result, "Texte avec espaces multiples");
    }

    #[test]
    fn expose_decodes_html_entities() {
        let raw = "A&nbsp;B &amp; C &lt;D&gt;";
        let result = normalize_expose_sommaire(raw, 500);
        assert_eq!(result, "A B & C <D>");
    }

    #[test]
    fn expose_truncates_with_ellipsis() {
        let raw = "Cet amendement vise à modifier le texte pour clarifier la situation.";
        let result = normalize_expose_sommaire(raw, 20);
        assert!(result.ends_with('…'));
        assert!(result.chars().count() <= 21); // 20 + ellipsis
    }

    #[test]
    fn expose_empty_input() {
        assert_eq!(normalize_expose_sommaire("", 500), "");
        assert_eq!(normalize_expose_sommaire("   ", 500), "");
        assert_eq!(normalize_expose_sommaire("<br/>", 500), "");
    }

    #[test]
    fn expose_no_truncation_when_short() {
        let raw = "Court texte.";
        assert_eq!(normalize_expose_sommaire(raw, 500), "Court texte.");
    }

    #[test]
    fn expose_decodes_numeric_hex_entities() {
        let raw = "pr&#x00E9;cis&#x00E9;ment";
        assert_eq!(normalize_expose_sommaire(raw, 500), "précisément");
    }

    #[test]
    fn expose_decodes_numeric_decimal_entities() {
        let raw = "pr&#233;cis&#233;ment";
        assert_eq!(normalize_expose_sommaire(raw, 500), "précisément");
    }

    // ─── normalize_profession_label ────────────────────────────────────────
    #[test]
    fn profession_strips_code_prefix() {
        assert_eq!(normalize_profession_label("(35) — Professeur"), "Professeur");
        assert_eq!(normalize_profession_label("(12)-Avocat"), "Avocat");
    }

    #[test]
    fn profession_keeps_plain_label() {
        assert_eq!(normalize_profession_label("Ingénieur"), "Ingénieur");
    }

    // ─── normalize_sexe_label ──────────────────────────────────────────────
    #[test]
    fn sexe_normalizes_variants() {
        assert_eq!(normalize_sexe_label("M"), Some("Homme".to_string()));
        assert_eq!(normalize_sexe_label("Mme"), Some("Femme".to_string()));
        assert_eq!(normalize_sexe_label("Monsieur"), Some("Homme".to_string()));
        assert_eq!(normalize_sexe_label("Madame"), Some("Femme".to_string()));
    }

    #[test]
    fn sexe_empty_or_null() {
        assert_eq!(normalize_sexe_label(""), None);
        assert_eq!(normalize_sexe_label("null"), None);
    }

    // ─── parse_amendement ──────────────────────────────────────────────────
    #[test]
    fn parse_amendement_extracts_fields() {
        let json = serde_json::json!({
            "uid": "AMANR5L17PO123456-1",
            "identificatif": { "numero": "42" },
            "signataires": {
                "auteur": {
                    "acteurRef": "PA1234",
                    "typeAuteur": "Député"
                },
                "cosignataires": {
                    "acteurRef": ["PA5678", "PA9999"]
                }
            },
            "cycleDeVie": {
                "sort": "Adopté",
                "dateDepot": "2024-01-15",
                "dateSort": "2024-01-20"
            },
            "dossierRef": "DLR5L17N12345",
            "pointeurFragmentTexte": {
                "division": { "titre": "Art. 3" },
                "texteLegislatifRef": "PRJLANR5L17B12345",
                "missionVisee": { "libelleMission": "Travail", "missionRef": "MIS-REF-001" }
            },
            "exposeSommaire": "<p>Cet amendement <b>vise</b> à clarifier.</p>"
        });

        let result = parse_amendement(&json).expect("should parse");
        assert_eq!(result.id, "AMANR5L17PO123456-1");
        assert_eq!(result.numero, Some("42".to_string()));
        assert_eq!(result.auteur_id, Some("PA1234".to_string()));
        assert_eq!(result.auteur_type, Some("Député".to_string()));
        assert_eq!(result.cosignataires_ids, vec!["PA5678".to_string(), "PA9999".to_string()]);
        assert!(result.adopte);
        assert_eq!(result.date_depot, Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
        assert_eq!(result.date_sort, Some(NaiveDate::from_ymd_opt(2024, 1, 20).unwrap()));
        assert_eq!(result.article, Some("Art. 3".to_string()));
        assert_eq!(result.mission_visee, Some("Travail".to_string()));
        assert_eq!(result.mission_ref, Some("MIS-REF-001".to_string()));
        // Expose should be HTML-stripped
        assert_eq!(result.expose_sommaire, Some("Cet amendement vise à clarifier.".to_string()));
    }

    #[test]
    fn parse_amendement_missing_optional_fields() {
        let json = serde_json::json!({
            "uid": "AMD-MINIMAL"
        });
        let result = parse_amendement(&json).expect("should parse minimal");
        assert_eq!(result.id, "AMD-MINIMAL");
        assert_eq!(result.auteur_id, None);
        assert_eq!(result.cosignataires_ids, Vec::<String>::new());
        assert_eq!(result.expose_sommaire, None);
        assert!(!result.adopte);
    }

    #[test]
    fn parse_amendement_expose_from_contenu_auteur() {
        let json = serde_json::json!({
            "uid": "AMD-NESTED-EXPOSE",
            "corps": {
                "contenuAuteur": {
                    "exposeSommaire": "<p>Texte sous contenuAuteur.</p>"
                }
            }
        });
        let result = parse_amendement(&json).expect("should parse nested expose");
        assert_eq!(result.expose_sommaire, Some("Texte sous contenuAuteur.".to_string()));
    }
}
