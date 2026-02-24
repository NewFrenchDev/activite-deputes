use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use crate::models::{Depute, Dossier};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupPplIndex {
    pub version: u8,
    pub generated_at: String,
    pub total_groups: usize,
    pub total_ppl_links: usize,
    pub total_unique_ppl: usize,
    pub groups: Vec<GroupPplGroupIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupPplGroupIndexEntry {
    pub group_id: String,
    pub group_label: String,
    pub ppl_count: usize,
    pub authored_ppl_count: usize,
    pub cosigned_only_ppl_count: usize,
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupPplGroupShard {
    pub version: u8,
    pub generated_at: String,
    pub group_id: String,
    pub group_label: String,
    pub total_entries: usize,
    pub items: Vec<GroupPplItemSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupPplItemSummary {
    pub ppl_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legislature: Option<u8>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deposit_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    pub author_count: u16,
    pub cosigner_count: u16,
    pub total_signers_from_group: u16,
    pub has_author: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signer_names_preview: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signers_preview: Vec<SignerPreviewEntry>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerPreviewEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deputy_id: Option<String>,
    pub deputy_name: String,
}

#[derive(Debug, Clone)]
struct GroupPplItemBuilder {
    ppl_id: String,
    number: Option<String>,
    legislature: Option<u8>,
    title: String,
    deposit_date: Option<String>,
    source_url: Option<String>,
    author_signers: BTreeSet<String>,
    cosigner_signers: BTreeSet<String>,
    signer_preview_seen: BTreeSet<String>,
    signer_names_preview: Vec<String>,
    signers_preview: Vec<SignerPreviewEntry>,
}

impl GroupPplItemBuilder {
    fn from_dossier(d: &Dossier) -> Self {
        Self {
            ppl_id: d.id.clone(),
            number: d.numero.clone(),
            legislature: d
                .legislature
                .as_deref()
                .and_then(|s| s.trim().parse::<u8>().ok()),
            title: normalize_whitespace(&d.titre),
            deposit_date: d.date_depot.map(|x| x.to_string()),
            source_url: d.source_url.clone(),
            author_signers: BTreeSet::new(),
            cosigner_signers: BTreeSet::new(),
            signer_preview_seen: BTreeSet::new(),
            signer_names_preview: Vec::new(),
            signers_preview: Vec::new(),
        }
    }

    fn add_author(
        &mut self,
        key: String,
        deputy_id: Option<&str>,
        name: Option<&str>,
        preview_limit: usize,
    ) {
        self.author_signers.insert(key.clone());
        self.maybe_add_preview(key, deputy_id, name, preview_limit);
    }

    fn add_cosigner(
        &mut self,
        key: String,
        deputy_id: Option<&str>,
        name: Option<&str>,
        preview_limit: usize,
    ) {
        self.cosigner_signers.insert(key.clone());
        self.maybe_add_preview(key, deputy_id, name, preview_limit);
    }

    fn maybe_add_preview(
        &mut self,
        key: String,
        deputy_id: Option<&str>,
        name: Option<&str>,
        preview_limit: usize,
    ) {
        if self.signers_preview.len() >= preview_limit {
            return;
        }
        if !self.signer_preview_seen.insert(key) {
            return;
        }
        if let Some(name) = name {
            let n = normalize_whitespace(name);
            if !n.is_empty() {
                self.signer_names_preview.push(n.clone());
                self.signers_preview.push(SignerPreviewEntry {
                    deputy_id: deputy_id.map(normalize_actor_id),
                    deputy_name: n,
                });
            }
        }
    }

    fn build(self) -> GroupPplItemSummary {
        let mut all = self.author_signers.clone();
        all.extend(self.cosigner_signers.iter().cloned());

        let author_count = saturating_u16(self.author_signers.len());
        let cosigner_count = saturating_u16(self.cosigner_signers.len());
        let total_signers_from_group = saturating_u16(all.len());

        GroupPplItemSummary {
            ppl_id: self.ppl_id,
            number: self.number,
            legislature: self.legislature,
            title: self.title,
            deposit_date: self.deposit_date,
            source_url: self.source_url,
            author_count,
            cosigner_count,
            total_signers_from_group,
            has_author: author_count > 0,
            signer_names_preview: self.signer_names_preview,
            signers_preview: self.signers_preview,
        }
    }
}

#[derive(Debug, Clone)]
struct GroupBuilder {
    group_id: String,
    group_label: String,
    items: HashMap<String, GroupPplItemBuilder>,
}

impl GroupBuilder {
    fn new(group_id: String, group_label: String) -> Self {
        Self {
            group_id,
            group_label,
            items: HashMap::new(),
        }
    }

    fn item_mut(&mut self, dossier: &Dossier) -> &mut GroupPplItemBuilder {
        self.items
            .entry(dossier.id.clone())
            .or_insert_with(|| GroupPplItemBuilder::from_dossier(dossier))
    }

    fn to_shard(self, generated_at: &str) -> GroupPplGroupShard {
        let mut items: Vec<GroupPplItemSummary> = self.items.into_values().map(|b| b.build()).collect();
        items.sort_by(|a, b| {
            b.deposit_date
                .cmp(&a.deposit_date)
                .then_with(|| b.has_author.cmp(&a.has_author))
                .then_with(|| b.total_signers_from_group.cmp(&a.total_signers_from_group))
                .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        });

        GroupPplGroupShard {
            version: 1,
            generated_at: generated_at.to_string(),
            group_id: self.group_id,
            group_label: self.group_label,
            total_entries: items.len(),
            items,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeputyPplShard {
    pub version: u8,
    pub generated_at: String,
    pub deputy_id: String,
    pub deputy_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_label: Option<String>,
    pub total_entries: usize,
    pub authored_count: usize,
    pub cosigned_only_count: usize,
    pub items: Vec<DeputyPplItemSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeputyPplItemSummary {
    pub ppl_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legislature: Option<u8>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deposit_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    pub is_author: bool,
    pub is_cosigner: bool,
}

#[derive(Debug, Clone)]
struct DeputyPplItemBuilder {
    ppl_id: String,
    number: Option<String>,
    legislature: Option<u8>,
    title: String,
    deposit_date: Option<String>,
    source_url: Option<String>,
    is_author: bool,
    is_cosigner: bool,
}

impl DeputyPplItemBuilder {
    fn from_dossier(d: &Dossier) -> Self {
        Self {
            ppl_id: d.id.clone(),
            number: d.numero.clone(),
            legislature: d
                .legislature
                .as_deref()
                .and_then(|s| s.trim().parse::<u8>().ok()),
            title: normalize_whitespace(&d.titre),
            deposit_date: d.date_depot.map(|x| x.to_string()),
            source_url: d.source_url.clone(),
            is_author: false,
            is_cosigner: false,
        }
    }

    fn mark_author(&mut self) {
        self.is_author = true;
    }

    fn mark_cosigner(&mut self) {
        self.is_cosigner = true;
    }

    fn build(self) -> DeputyPplItemSummary {
        DeputyPplItemSummary {
            ppl_id: self.ppl_id,
            number: self.number,
            legislature: self.legislature,
            title: self.title,
            deposit_date: self.deposit_date,
            source_url: self.source_url,
            is_author: self.is_author,
            is_cosigner: self.is_cosigner,
        }
    }
}

#[derive(Debug, Clone)]
struct DeputyPplBuilder {
    deputy_id: String,
    deputy_name: String,
    group_id: Option<String>,
    group_label: Option<String>,
    items: HashMap<String, DeputyPplItemBuilder>,
}

impl DeputyPplBuilder {
    fn from_deputy(dep: &DeputyLite) -> Self {
        let group_id = if dep.has_group {
            Some(dep.group_id.clone())
        } else {
            None
        };
        let group_label = if dep.has_group {
            Some(dep.group_label.clone())
        } else {
            None
        };

        Self {
            deputy_id: dep.id.clone(),
            deputy_name: dep.full_name.clone(),
            group_id,
            group_label,
            items: HashMap::new(),
        }
    }

    fn item_mut(&mut self, dossier: &Dossier) -> &mut DeputyPplItemBuilder {
        self.items
            .entry(dossier.id.clone())
            .or_insert_with(|| DeputyPplItemBuilder::from_dossier(dossier))
    }

    fn to_shard(self, generated_at: &str) -> DeputyPplShard {
        let mut items: Vec<DeputyPplItemSummary> = self.items.into_values().map(|b| b.build()).collect();
        items.sort_by(|a, b| {
            b.deposit_date
                .cmp(&a.deposit_date)
                .then_with(|| b.is_author.cmp(&a.is_author))
                .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        });

        let authored_count = items.iter().filter(|i| i.is_author).count();
        let cosigned_only_count = items.iter().filter(|i| !i.is_author && i.is_cosigner).count();

        DeputyPplShard {
            version: 1,
            generated_at: generated_at.to_string(),
            deputy_id: self.deputy_id,
            deputy_name: self.deputy_name,
            group_id: self.group_id,
            group_label: self.group_label,
            total_entries: items.len(),
            authored_count,
            cosigned_only_count,
            items,
        }
    }
}

#[derive(Debug, Clone)]
struct DeputyLite {
    id: String,
    full_name: String,
    group_id: String,
    group_label: String,
    has_group: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GroupPplDebugSummary {
    version: u8,
    generated_at: String,
    deputes_total: usize,
    deputes_with_group: usize,
    deputes_without_group: usize,
    dossiers_total: usize,
    ppl_detected: usize,
    ppl_origin_assemblee: usize,
    ppl_origin_senat: usize,
    ppl_origin_unknown: usize,
    ppl_skipped_non_assemblee: usize,
    ppl_with_signers: usize,
    unique_ppl_retained: usize,
    total_author_signers_seen: usize,
    total_cosigners_seen: usize,
    signers_resolved_to_group: usize,
    signers_unresolved_deputy_not_found: usize,
    signers_unresolved_deputy_without_group: usize,
    unknown_bucket_ppl_entries: usize,
    unknown_bucket_authored_entries: usize,
    unknown_bucket_signers_seen: usize,
    unresolved_by_legislature: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GroupPplUnresolvedSignerSample {
    dossier_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    legislature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deposit_date: Option<String>,
    signer_role: String,
    signer_id: String,
    cause: String,
    titre: String,
}


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GroupPplDeputyIdAuditSummary {
    version: u8,
    generated_at: String,
    deputes_total: usize,
    unique_pa_ids: usize,
    duplicate_pa_id_entries: usize,
    names_with_multiple_pa_ids: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GroupPplDeputyIdDuplicateEntry {
    normalized_name: String,
    display_names: Vec<String>,
    pa_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GroupPplDeputyMandateDebugEntry {
    deputy_id: String,
    full_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_label: Option<String>,
    mandat_assemblee_episode_count: usize,
    mandat_assemblee_episodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GroupPplDeputyIdAuditReport {
    summary: GroupPplDeputyIdAuditSummary,
    #[serde(default)]
    duplicate_pa_ids: Vec<String>,
    #[serde(default)]
    names_with_multiple_pa_ids: Vec<GroupPplDeputyIdDuplicateEntry>,
    #[serde(default)]
    deputies: Vec<GroupPplDeputyMandateDebugEntry>,
}

pub fn write_group_ppl_json(
    deputes: &[Depute],
    dossiers: &HashMap<String, Dossier>,
    data_root: &Path,
    generated_at_iso: &str,
) -> Result<()> {
    let out_dir = data_root.join("positions-groupes").join("ppl");
    let groups_dir = out_dir.join("groups");
    std::fs::create_dir_all(&groups_dir)?;

    let deputy_out_dir = data_root.join("positions-deputes").join("ppl");
    std::fs::create_dir_all(&deputy_out_dir)?;

    let mut deputes_with_group = 0usize;
    let mut deputes_without_group = 0usize;
    let mut deputy_map: HashMap<String, DeputyLite> = HashMap::new();
    for d in deputes {
        let raw_group_id = d.groupe_id.clone().or_else(|| d.groupe_abrev.clone());
        let raw_group_label = d.groupe_abrev.clone().or_else(|| d.groupe_nom.clone());
        let has_group = raw_group_id.is_some() || raw_group_label.is_some();
        if has_group {
            deputes_with_group += 1;
        } else {
            deputes_without_group += 1;
        }

        let group_id = raw_group_id.unwrap_or_else(|| "INCONNU".to_string());
        let group_label = raw_group_label.unwrap_or_else(|| "Inconnu".to_string());
        let full_name = format!("{} {}", d.prenom, d.nom).trim().to_string();
        let lite = DeputyLite {
            id: d.id.clone(),
            full_name,
            group_id,
            group_label,
            has_group,
        };

        // On indexe en exact + normalisé pour éviter les ratés triviaux (casse/espaces).
        deputy_map.entry(d.id.clone()).or_insert_with(|| lite.clone());
        deputy_map
            .entry(normalize_actor_id(&d.id))
            .or_insert_with(|| lite.clone());
    }

    let mut groups_map: HashMap<String, GroupBuilder> = HashMap::new();
    let mut deputy_ppl_map: HashMap<String, DeputyPplBuilder> = HashMap::new();
    let mut unique_ppl_ids: BTreeSet<String> = BTreeSet::new();

    let mut dossiers_total = 0usize;
    let mut ppl_detected = 0usize;
    let mut ppl_origin_assemblee = 0usize;
    let mut ppl_origin_senat = 0usize;
    let mut ppl_origin_unknown = 0usize;
    let mut ppl_skipped_non_assemblee = 0usize;
    let mut ppl_with_signers = 0usize;

    let mut total_author_signers_seen = 0usize;
    let mut total_cosigners_seen = 0usize;
    let mut signers_resolved_to_group = 0usize;
    let mut signers_unresolved_deputy_not_found = 0usize;
    let mut signers_unresolved_deputy_without_group = 0usize;

    let mut unknown_bucket_ppl_entries: BTreeSet<String> = BTreeSet::new();
    let mut unknown_bucket_authored_entries: BTreeSet<String> = BTreeSet::new();
    let mut unknown_bucket_signers_seen = 0usize;
    let mut unresolved_by_legislature: BTreeMap<String, usize> = BTreeMap::new();
    let mut unresolved_samples: Vec<GroupPplUnresolvedSignerSample> = Vec::new();
    const UNRESOLVED_SAMPLE_LIMIT: usize = 200;

    for dossier in dossiers.values() {
        dossiers_total += 1;
        if !is_proposition_de_loi(dossier) {
            continue;
        }
        ppl_detected += 1;

        match dossier.origin_chamber.as_deref() {
            Some("assemblee") => {
                ppl_origin_assemblee += 1;
            }
            Some("senat") => {
                ppl_origin_senat += 1;
                ppl_skipped_non_assemblee += 1;
                continue;
            }
            _ => {
                ppl_origin_unknown += 1;
                // V1.1: on garde les origines inconnues pour ne pas perdre de signal,
                // mais on les trace dans le debug summary.
            }
        }

        // On ne crée pas d'entrée sans signataire connu, pour éviter les faux liens.
        if dossier.auteur_id.is_none() && dossier.cosignataires_ids.is_empty() {
            continue;
        }
        ppl_with_signers += 1;

        unique_ppl_ids.insert(dossier.id.clone());

        if let Some(auteur_id) = dossier.auteur_id.as_deref() {
            total_author_signers_seen += 1;
            let dep = lookup_deputy(&deputy_map, auteur_id);
            let (group_id, group_label) = match dep {
                Some(d) if d.has_group => {
                    signers_resolved_to_group += 1;
                    (d.group_id.clone(), d.group_label.clone())
                }
                Some(_) => {
                    signers_unresolved_deputy_without_group += 1;
                    unknown_bucket_signers_seen += 1;
                    unknown_bucket_ppl_entries.insert(dossier.id.clone());
                    unknown_bucket_authored_entries.insert(dossier.id.clone());
                    *unresolved_by_legislature
                        .entry(dossier.legislature.clone().unwrap_or_else(|| "?".to_string()))
                        .or_insert(0) += 1;
                    if unresolved_samples.len() < UNRESOLVED_SAMPLE_LIMIT {
                        unresolved_samples.push(GroupPplUnresolvedSignerSample {
                            dossier_id: dossier.id.clone(),
                            legislature: dossier.legislature.clone(),
                            deposit_date: dossier.date_depot.map(|d| d.to_string()),
                            signer_role: "author".to_string(),
                            signer_id: auteur_id.trim().to_string(),
                            cause: "deputy_without_group".to_string(),
                            titre: dossier.titre.clone(),
                        });
                    }
                    ("INCONNU".to_string(), "Inconnu".to_string())
                }
                None => {
                    signers_unresolved_deputy_not_found += 1;
                    unknown_bucket_signers_seen += 1;
                    unknown_bucket_ppl_entries.insert(dossier.id.clone());
                    unknown_bucket_authored_entries.insert(dossier.id.clone());
                    *unresolved_by_legislature
                        .entry(dossier.legislature.clone().unwrap_or_else(|| "?".to_string()))
                        .or_insert(0) += 1;
                    if unresolved_samples.len() < UNRESOLVED_SAMPLE_LIMIT {
                        unresolved_samples.push(GroupPplUnresolvedSignerSample {
                            dossier_id: dossier.id.clone(),
                            legislature: dossier.legislature.clone(),
                            deposit_date: dossier.date_depot.map(|d| d.to_string()),
                            signer_role: "author".to_string(),
                            signer_id: auteur_id.trim().to_string(),
                            cause: "deputy_not_found".to_string(),
                            titre: dossier.titre.clone(),
                        });
                    }
                    ("INCONNU".to_string(), "Inconnu".to_string())
                }
            };

            let gb = groups_map
                .entry(group_id.clone())
                .or_insert_with(|| GroupBuilder::new(group_id, group_label));
            let item = gb.item_mut(dossier);
            let signer_id_key = normalize_actor_id(auteur_id);
            let signer_label = dep.map(|d| d.full_name.as_str());
            item.add_author(
                signer_key(&signer_id_key),
                dep.map(|d| d.id.as_str()),
                signer_label,
                6,
            );
            if let Some(d) = dep {
                let db = deputy_ppl_map
                    .entry(d.id.clone())
                    .or_insert_with(|| DeputyPplBuilder::from_deputy(d));
                db.item_mut(dossier).mark_author();
            }
        }

        let auteur_id_norm = dossier.auteur_id.as_deref().map(normalize_actor_id);
        let mut seen_cos: BTreeSet<String> = BTreeSet::new();
        for cos_id in &dossier.cosignataires_ids {
            let cos_id_norm = normalize_actor_id(cos_id);
            if !seen_cos.insert(cos_id_norm.clone()) {
                continue;
            }
            if auteur_id_norm.as_deref() == Some(cos_id_norm.as_str())
            {
                continue;
            }

            total_cosigners_seen += 1;
            let dep = lookup_deputy(&deputy_map, &cos_id_norm);
            let (group_id, group_label) = match dep {
                Some(d) if d.has_group => {
                    signers_resolved_to_group += 1;
                    (d.group_id.clone(), d.group_label.clone())
                }
                Some(_) => {
                    signers_unresolved_deputy_without_group += 1;
                    unknown_bucket_signers_seen += 1;
                    unknown_bucket_ppl_entries.insert(dossier.id.clone());
                    *unresolved_by_legislature
                        .entry(dossier.legislature.clone().unwrap_or_else(|| "?".to_string()))
                        .or_insert(0) += 1;
                    if unresolved_samples.len() < UNRESOLVED_SAMPLE_LIMIT {
                        unresolved_samples.push(GroupPplUnresolvedSignerSample {
                            dossier_id: dossier.id.clone(),
                            legislature: dossier.legislature.clone(),
                            deposit_date: dossier.date_depot.map(|d| d.to_string()),
                            signer_role: "cosigner".to_string(),
                            signer_id: cos_id_norm.clone(),
                            cause: "deputy_without_group".to_string(),
                            titre: dossier.titre.clone(),
                        });
                    }
                    ("INCONNU".to_string(), "Inconnu".to_string())
                }
                None => {
                    signers_unresolved_deputy_not_found += 1;
                    unknown_bucket_signers_seen += 1;
                    unknown_bucket_ppl_entries.insert(dossier.id.clone());
                    *unresolved_by_legislature
                        .entry(dossier.legislature.clone().unwrap_or_else(|| "?".to_string()))
                        .or_insert(0) += 1;
                    if unresolved_samples.len() < UNRESOLVED_SAMPLE_LIMIT {
                        unresolved_samples.push(GroupPplUnresolvedSignerSample {
                            dossier_id: dossier.id.clone(),
                            legislature: dossier.legislature.clone(),
                            deposit_date: dossier.date_depot.map(|d| d.to_string()),
                            signer_role: "cosigner".to_string(),
                            signer_id: cos_id_norm.clone(),
                            cause: "deputy_not_found".to_string(),
                            titre: dossier.titre.clone(),
                        });
                    }
                    ("INCONNU".to_string(), "Inconnu".to_string())
                }
            };

            let gb = groups_map
                .entry(group_id.clone())
                .or_insert_with(|| GroupBuilder::new(group_id, group_label));
            let item = gb.item_mut(dossier);
            let signer_label = dep.map(|d| d.full_name.as_str());
            item.add_cosigner(
                signer_key(&cos_id_norm),
                dep.map(|d| d.id.as_str()),
                signer_label,
                6,
            );
            if let Some(d) = dep {
                let db = deputy_ppl_map
                    .entry(d.id.clone())
                    .or_insert_with(|| DeputyPplBuilder::from_deputy(d));
                db.item_mut(dossier).mark_cosigner();
            }
        }
    }

    let mut group_builders: Vec<GroupBuilder> = groups_map.into_values().collect();
    group_builders.sort_by(|a, b| {
        a.group_label
            .to_lowercase()
            .cmp(&b.group_label.to_lowercase())
            .then_with(|| a.group_id.cmp(&b.group_id))
    });

    let mut index_entries = Vec::with_capacity(group_builders.len());
    let mut total_links = 0usize;

    for gb in group_builders {
        let file_name = format!("{}.json", safe_file_stem(&gb.group_id));
        let rel_file = format!("groups/{file_name}");
        let shard = gb.to_shard(generated_at_iso);
        total_links += shard.total_entries;

        let (authored_ppl_count, cosigned_only_ppl_count) = count_authored_vs_cosigned_only(&shard.items);

        write_minified_json(&out_dir.join(&rel_file), &shard)?;

        index_entries.push(GroupPplGroupIndexEntry {
            group_id: shard.group_id,
            group_label: shard.group_label,
            ppl_count: shard.total_entries,
            authored_ppl_count,
            cosigned_only_ppl_count,
            file: rel_file,
        });
    }

    let mut deputy_builders: Vec<DeputyPplBuilder> = deputy_ppl_map.into_values().collect();
    deputy_builders.sort_by(|a, b| {
        a.deputy_name
            .to_lowercase()
            .cmp(&b.deputy_name.to_lowercase())
            .then_with(|| a.deputy_id.cmp(&b.deputy_id))
    });

    for db in deputy_builders {
        let file_name = format!("{}.json", safe_file_stem(&db.deputy_id));
        let shard = db.to_shard(generated_at_iso);
        write_minified_json(&deputy_out_dir.join(file_name), &shard)?;
    }

    let index = GroupPplIndex {
        version: 1,
        generated_at: generated_at_iso.to_string(),
        total_groups: index_entries.len(),
        total_ppl_links: total_links,
        total_unique_ppl: unique_ppl_ids.len(),
        groups: index_entries,
    };

    write_minified_json(&out_dir.join("index.json"), &index)?;

    let debug_summary = GroupPplDebugSummary {
        version: 1,
        generated_at: generated_at_iso.to_string(),
        deputes_total: deputes.len(),
        deputes_with_group,
        deputes_without_group,
        dossiers_total,
        ppl_detected,
        ppl_origin_assemblee,
        ppl_origin_senat,
        ppl_origin_unknown,
        ppl_skipped_non_assemblee,
        ppl_with_signers,
        unique_ppl_retained: unique_ppl_ids.len(),
        total_author_signers_seen,
        total_cosigners_seen,
        signers_resolved_to_group,
        signers_unresolved_deputy_not_found,
        signers_unresolved_deputy_without_group,
        unknown_bucket_ppl_entries: unknown_bucket_ppl_entries.len(),
        unknown_bucket_authored_entries: unknown_bucket_authored_entries.len(),
        unknown_bucket_signers_seen,
        unresolved_by_legislature,
    };
    write_minified_json(&out_dir.join("debug_summary.json"), &debug_summary)?;
    write_minified_json(&out_dir.join("unresolved_signers_sample.json"), &unresolved_samples)?;

    let deputy_id_audit = build_deputy_id_audit_report(deputes, generated_at_iso);
    write_minified_json(&out_dir.join("deputes_id_audit.json"), &deputy_id_audit)?;

    Ok(())
}

fn build_deputy_id_audit_report(
    deputes: &[Depute],
    generated_at_iso: &str,
) -> GroupPplDeputyIdAuditReport {
    let mut pa_ids_seen: BTreeSet<String> = BTreeSet::new();
    let mut duplicate_pa_ids: BTreeSet<String> = BTreeSet::new();
    let mut names_to_ids: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut names_display: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    let mut deputies: Vec<GroupPplDeputyMandateDebugEntry> = deputes
        .iter()
        .map(|d| {
            let pa_id = normalize_actor_id(&d.id);
            if !pa_ids_seen.insert(pa_id.clone()) {
                duplicate_pa_ids.insert(pa_id.clone());
            }

            let full_name = normalize_whitespace(&format!("{} {}", d.prenom, d.nom));
            let norm_name = full_name.to_lowercase();
            names_to_ids.entry(norm_name.clone()).or_default().insert(pa_id);
            names_display.entry(norm_name).or_default().insert(full_name.clone());

            GroupPplDeputyMandateDebugEntry {
                deputy_id: d.id.clone(),
                full_name,
                group_label: d.groupe_abrev.clone().or_else(|| d.groupe_nom.clone()),
                mandat_assemblee_episode_count: d.mandat_assemblee_episodes.len(),
                mandat_assemblee_episodes: format_mandat_episode_labels(d),
            }
        })
        .collect();

    deputies.sort_by(|a, b| {
        a.full_name
            .to_lowercase()
            .cmp(&b.full_name.to_lowercase())
            .then_with(|| a.deputy_id.cmp(&b.deputy_id))
    });

    let mut names_with_multiple_pa_ids: Vec<GroupPplDeputyIdDuplicateEntry> = names_to_ids
        .into_iter()
        .filter_map(|(normalized_name, ids)| {
            if ids.len() <= 1 {
                return None;
            }
            let display_names = names_display
                .remove(&normalized_name)
                .map(|s| s.into_iter().collect())
                .unwrap_or_else(Vec::new);
            Some(GroupPplDeputyIdDuplicateEntry {
                normalized_name,
                display_names,
                pa_ids: ids.into_iter().collect(),
            })
        })
        .collect();
    names_with_multiple_pa_ids.sort_by(|a, b| a.normalized_name.cmp(&b.normalized_name));

    GroupPplDeputyIdAuditReport {
        summary: GroupPplDeputyIdAuditSummary {
            version: 1,
            generated_at: generated_at_iso.to_string(),
            deputes_total: deputes.len(),
            unique_pa_ids: pa_ids_seen.len(),
            duplicate_pa_id_entries: duplicate_pa_ids.len(),
            names_with_multiple_pa_ids: names_with_multiple_pa_ids.len(),
        },
        duplicate_pa_ids: duplicate_pa_ids.into_iter().collect(),
        names_with_multiple_pa_ids,
        deputies,
    }
}

fn format_mandat_episode_labels(dep: &Depute) -> Vec<String> {
    dep.mandat_assemblee_episodes
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let end = e
                .date_fin
                .map(|d| d.to_string())
                .unwrap_or_else(|| "en cours".to_string());
            format!("Mandat AN épisode {}: {} → {}", i + 1, e.date_debut, end)
        })
        .collect()
}

fn count_authored_vs_cosigned_only(items: &[GroupPplItemSummary]) -> (usize, usize) {
    let mut authored = 0usize;
    let mut cosigned_only = 0usize;
    for item in items {
        if item.has_author {
            authored += 1;
        } else {
            cosigned_only += 1;
        }
    }
    (authored, cosigned_only)
}

fn normalize_actor_id(raw: &str) -> String {
    raw.trim().to_ascii_uppercase()
}

fn lookup_deputy<'a>(deputy_map: &'a HashMap<String, DeputyLite>, signer_id: &str) -> Option<&'a DeputyLite> {
    deputy_map
        .get(signer_id)
        .or_else(|| deputy_map.get(&normalize_actor_id(signer_id)))
}

fn signer_key(dep_id: &str) -> String {
    format!("id:{}", normalize_actor_id(dep_id))
}

fn write_minified_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let bytes = serde_json::to_vec(value)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn saturating_u16(v: usize) -> u16 {
    if v > u16::MAX as usize {
        u16::MAX
    } else {
        v as u16
    }
}

fn safe_file_stem(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else {
            out.push('-');
        }
    }

    let mut collapsed = String::with_capacity(out.len());
    let mut prev_dash = false;
    for c in out.chars() {
        if c == '-' {
            if !prev_dash {
                collapsed.push(c);
            }
            prev_dash = true;
        } else {
            collapsed.push(c);
            prev_dash = false;
        }
    }

    let stem = collapsed.trim_matches('-').to_string();
    if stem.is_empty() {
        "group".to_string()
    } else {
        stem
    }
}

fn is_proposition_de_loi(d: &Dossier) -> bool {
    if let Some(statut) = d.statut.as_deref() {
        if is_ppl_label(statut) {
            return true;
        }
    }
    if let Some(nature) = d.nature.as_deref() {
        if is_ppl_label(nature) {
            return true;
        }
    }
    is_ppl_label(&d.titre)
}

fn is_ppl_label(raw: &str) -> bool {
    let s = raw.trim().to_lowercase();
    // On cible la famille "Proposition de loi ..." (inclut organique/constitutionnelle).
    // On exclut explicitement les résolutions pour éviter l'ambiguïté en V1.
    s.starts_with("proposition de loi") && !s.starts_with("proposition de résolution")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn detects_ppl_labels() {
        let d = Dossier {
            id: "D1".into(),
            titre: "Proposition de loi visant à ...".into(),
            date_depot: None,
            statut: None,
            legislature: Some("17".into()),
            nature: None,
            numero: Some("1234".into()),
            auteur_id: Some("PA1".into()),
            cosignataires_ids: vec!["PA2".into()],
            source_url: None,
            origin_chamber: Some("assemblee".into()),
            initiateur_organe_ref: None,
        };
        assert!(is_proposition_de_loi(&d));
    }

    #[test]
    fn writes_minimal_shards() {
        let deputes = vec![
            Depute {
                id: "PA1".into(), nom: "Dupont".into(), prenom: "Alice".into(),
                date_naissance: None, sexe: None, pays_naissance: None, profession: None,
                dept_code: None, dept_nom: None, circo: None,
                mandat_debut: None, mandat_fin: None, mandat_debut_legislature: None,
                mandat_assemblee_episodes: vec![],
                groupe_id: Some("POGRP1".into()), groupe_abrev: Some("GRP1".into()), groupe_nom: Some("Groupe 1".into()),
                parti_id: None, parti_nom: None, email_assemblee: None, site_web: None,
                sites_web: vec![], sites_web_sources: vec![], telephones: vec![], uri_hatvp: None,
            },
            Depute {
                id: "PA2".into(), nom: "Martin".into(), prenom: "Bob".into(),
                date_naissance: None, sexe: None, pays_naissance: None, profession: None,
                dept_code: None, dept_nom: None, circo: None,
                mandat_debut: None, mandat_fin: None, mandat_debut_legislature: None,
                mandat_assemblee_episodes: vec![],
                groupe_id: Some("POGRP1".into()), groupe_abrev: Some("GRP1".into()), groupe_nom: Some("Groupe 1".into()),
                parti_id: None, parti_nom: None, email_assemblee: None, site_web: None,
                sites_web: vec![], sites_web_sources: vec![], telephones: vec![], uri_hatvp: None,
            },
        ];

        let mut dossiers = HashMap::new();
        dossiers.insert("D1".into(), Dossier {
            id: "D1".into(),
            titre: "Proposition de loi test".into(),
            date_depot: Some(NaiveDate::from_ymd_opt(2026, 2, 24).unwrap()),
            statut: None,
            legislature: Some("17".into()),
            nature: None,
            numero: Some("1234".into()),
            auteur_id: Some("PA1".into()),
            cosignataires_ids: vec!["PA2".into()],
            source_url: None,
            origin_chamber: Some("assemblee".into()),
            initiateur_organe_ref: None,
        });

        let tmp = std::env::temp_dir().join("group_ppl_v1_pipe_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        write_group_ppl_json(&deputes, &dossiers, &tmp, "2026-02-24T04:00:00Z").unwrap();
        assert!(tmp.join("positions-groupes/ppl/index.json").exists());
        assert!(tmp.join("positions-deputes/ppl/pa1.json").exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
