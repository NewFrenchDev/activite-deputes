use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Raw AN JSON structures ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DeputesRoot {
    pub export: DeputesExport,
}

#[derive(Debug, Deserialize)]
pub struct DeputesExport {
    pub acteurs: ActeursWrapper,
    pub organes: OrganesWrapper,
}

#[derive(Debug, Deserialize)]
pub struct ActeursWrapper {
    pub acteur: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct OrganesWrapper {
    pub organe: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ScrutinsRoot {
    pub scrutins: Option<ScrutinsWrapper>,
}

#[derive(Debug, Deserialize)]
pub struct ScrutinsWrapper {
    pub scrutin: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AmendementsRoot {
    pub amendements: Option<AmendementsWrapper>,
}

#[derive(Debug, Deserialize)]
pub struct AmendementsWrapper {
    pub amendement: Vec<serde_json::Value>,
}

// ─── Normalized models ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SiteWebSource {
    pub type_libelle: Option<String>,
    pub val_elec: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MandatAssembleeEpisode {
    pub date_debut: NaiveDate,
    #[serde(default)]
    pub date_fin: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Depute {
    pub id: String,
    pub nom: String,
    pub prenom: String,
    pub date_naissance: Option<NaiveDate>,
    #[serde(default)]
    pub sexe: Option<String>,
    pub pays_naissance: Option<String>,
    pub profession: Option<String>,
    pub dept_code: Option<String>,
    pub dept_nom: Option<String>,
    pub circo: Option<String>,
    pub mandat_debut: Option<NaiveDate>,
    #[serde(default)]
    pub mandat_fin: Option<NaiveDate>,
    #[serde(default)]
    pub mandat_debut_legislature: Option<NaiveDate>,
    #[serde(default)]
    pub mandat_assemblee_episodes: Vec<MandatAssembleeEpisode>,
    pub groupe_id: Option<String>,
    pub groupe_abrev: Option<String>,
    pub groupe_nom: Option<String>,
    pub parti_id: Option<String>,
    pub parti_nom: Option<String>,
    pub email_assemblee: Option<String>,
    pub site_web: Option<String>,
    #[serde(default)]
    pub sites_web: Vec<String>,
    #[serde(default)]
    pub sites_web_sources: Vec<SiteWebSource>,
    #[serde(default)]
    pub telephones: Vec<String>,
    pub uri_hatvp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organe {
    pub id: String,
    pub code_type: String,
    pub libelle: String,
    pub abrev: Option<String>,
    pub couleur: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scrutin {
    pub id: String,
    pub numero: u32,
    pub titre: String,
    pub date: Option<NaiveDate>,
    pub sort: Option<String>,
    pub dossier_ref: Option<String>,
    pub votes: HashMap<String, VotePosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VotePosition {
    Pour,
    Contre,
    Abstention,
    NonVotant,
    Absent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Amendement {
    pub id: String,
    pub numero: Option<String>,
    pub auteur_id: Option<String>,
    #[serde(default)]
    pub auteur_type: Option<String>,
    pub cosignataires_ids: Vec<String>,
    pub sort: Option<String>,
    /// Date best-effort (fallback) utilisée par les agrégats existants.
    pub date: Option<NaiveDate>,
    /// Dates structurées (si présentes) — utiles pour une timeline complète.
    #[serde(default)]
    pub date_depot: Option<NaiveDate>,
    #[serde(default)]
    pub date_circulation: Option<NaiveDate>,
    #[serde(default)]
    pub date_examen: Option<NaiveDate>,
    #[serde(default)]
    pub date_sort: Option<NaiveDate>,
    pub dossier_ref: Option<String>,
    pub article: Option<String>,
    pub texte_ref: Option<String>,
    pub adopte: bool,
    #[serde(default)]
    pub mission_visee: Option<String>,
    #[serde(default)]
    pub mission_ref: Option<String>,
    #[serde(default)]
    pub expose_sommaire: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dossier {
    pub id: String,
    pub titre: String,
    pub date_depot: Option<NaiveDate>,
    pub statut: Option<String>,
    pub legislature: Option<String>,
    #[serde(default)]
    pub nature: Option<String>,
    #[serde(default)]
    pub numero: Option<String>,
    #[serde(default)]
    pub auteur_id: Option<String>,
    #[serde(default)]
    pub cosignataires_ids: Vec<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub origin_chamber: Option<String>, // "assemblee" | "senat" (best effort)
    #[serde(default)]
    pub initiateur_organe_ref: Option<String>,
}

// ─── Aggregated output ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeputeStats {
    pub deputy_id: String,
    pub nom: String,
    pub prenom: String,
    pub groupe_abrev: Option<String>,
    pub groupe_nom: Option<String>,
    pub parti_rattachement: Option<String>,
    pub dept: Option<String>,
    pub circo: Option<String>,
    pub mandat_debut: Option<NaiveDate>,
    #[serde(default)]
    pub mandat_fin: Option<NaiveDate>,
    #[serde(default)]
    pub mandat_debut_legislature: Option<NaiveDate>,
    #[serde(default)]
    pub mandat_assemblee_episodes: Vec<MandatAssembleeEpisode>,
    pub date_naissance: Option<NaiveDate>,
    #[serde(default)]
    pub sexe: Option<String>,
    pub pays_naissance: Option<String>,
    pub profession: Option<String>,
    pub email_assemblee: Option<String>,
    pub site_web: Option<String>,
    #[serde(default)]
    pub sites_web: Vec<String>,
    #[serde(default)]
    pub sites_web_sources: Vec<SiteWebSource>,
    #[serde(default)]
    pub telephones: Vec<String>,
    pub uri_hatvp: Option<String>,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub scrutins_eligibles: u32,
    pub votes_exprimes: u32,
    pub non_votant: u32,
    pub absent: u32,
    pub participation_rate: f64,
    pub pour_count: u32,
    pub contre_count: u32,
    pub abst_count: u32,
    pub amd_authored: u32,
    pub amd_adopted: u32,
    pub amd_adoption_rate: Option<f64>,
    pub amd_cosigned: u32,
    pub interventions_count: u32,
    pub interventions_chars: u32,
    pub top_dossiers: Vec<DossierScore>,
    #[serde(default)]
    pub top_cosignataires: Vec<TopCosignataire>,
    #[serde(default)]
    pub cosign_network: Option<CosignNetworkStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DossierScore {
    pub dossier_id: String,
    pub titre: String,
    pub votes: u32,
    pub amendements: u32,
    pub interventions: u32,
    pub score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopCosignataire {
    pub deputy_id: String,
    pub nom: String,
    pub prenom: String,
    pub groupe_abrev: Option<String>,
    pub co_signed_count: u32,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CosignNetworkStats {
    pub total_cosignatures: u32,
    pub unique_cosignataires: u32,
    pub in_group_count: u32,
    pub out_group_count: u32,
    #[serde(default)]
    pub in_group: Vec<CosignPeer>,
    #[serde(default)]
    pub out_group_groups: Vec<CosignGroupBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CosignPeer {
    pub deputy_id: String,
    pub nom: String,
    pub prenom: String,
    pub groupe_abrev: Option<String>,
    pub groupe_nom: Option<String>,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CosignGroupBucket {
    pub groupe_abrev: Option<String>,
    pub groupe_nom: Option<String>,
    pub count_total: u32,
    #[serde(default)]
    pub members: Vec<CosignPeer>,
}

// ─── Full parsed dataset ───────────────────────────────────────────────────

pub struct RawDataset {
    pub deputes: Vec<Depute>,
    pub organes: HashMap<String, Organe>,
    pub scrutins: Vec<Scrutin>,
    pub amendements: Vec<Amendement>,
    pub dossiers: HashMap<String, Dossier>,
}
