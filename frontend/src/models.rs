use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Status {
    pub last_update: String,
    pub last_update_readable: String,
    pub legislature: u32,
    pub sources: Vec<SourceInfo>,
    pub counts: Counts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceInfo {
    pub key: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Counts {
    pub deputes: usize,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeputeInfo {
    pub id: String,
    pub nom: String,
    pub prenom: String,
    pub date_naissance: Option<NaiveDate>,
    #[serde(default)]
    pub sexe: Option<String>,
    pub pays_naissance: Option<String>,
    pub profession: Option<String>,
    pub groupe_abrev: Option<String>,
    pub groupe_nom: Option<String>,
    pub parti_nom: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

// ─────────────────────────────────────────────────────────────────────────────
// Amendements — calendrier (shards par mois)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AmendementsIndex {
    pub schema_version: u32,
    pub generated_at: String,
    pub months: Vec<AmendementsMonthMeta>,
    pub undated_count: usize,
    pub undated_file: String,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AmendementsMonthMeta {
    pub month: String,
    pub days: usize,
    pub events: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AmendementsMonthFile {
    pub schema_version: u32,
    pub month: String,
    /// days[YYYY-MM-DD] = [events...]
    pub days: HashMap<String, Vec<AmendementEvent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AmendementEvent {
    /// Type: DEPOT | CIRCULATION | EXAMEN | SORT
    pub t: String,
    /// ID amendement
    pub id: String,
    /// Numéro
    #[serde(default)]
    pub n: Option<String>,
    /// Auteur ID (député)
    #[serde(default)]
    pub aid: Option<String>,
    /// Type d'auteur (Député, Groupe, etc.)
    #[serde(default)]
    pub aty: Option<String>,
    /// Cosignataires IDs (list of deputy IDs)
    #[serde(default)]
    pub cos: Vec<String>,
    /// Dossier ID
    #[serde(default)]
    pub did: Option<String>,
    /// Article (ex: "Art. 3")
    #[serde(default)]
    pub art: Option<String>,
    /// Sort (uniquement pour t=SORT)
    #[serde(default)]
    pub s: Option<String>,
    /// Adopté (uniquement pour t=SORT)
    #[serde(default)]
    pub ok: bool,
    /// Mission visée
    #[serde(default)]
    pub mis: Option<String>,
    /// Mission ref
    #[serde(default)]
    pub mref: Option<String>,
    /// Exposé sommaire
    #[serde(default)]
    pub exp: Option<String>,
}

pub type DossiersMin = HashMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DossierScore {
    pub dossier_id: String,
    pub titre: String,
    pub votes: u32,
    pub amendements: u32,
    pub interventions: u32,
    pub score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TopCosignataire {
    pub deputy_id: String,
    pub nom: String,
    pub prenom: String,
    pub groupe_abrev: Option<String>,
    #[serde(alias = "co_signature_count")]
    pub co_signed_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CosignPeer {
    pub deputy_id: String,
    pub nom: String,
    pub prenom: String,
    pub groupe_abrev: Option<String>,
    pub groupe_nom: Option<String>,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CosignGroupBucket {
    pub groupe_abrev: Option<String>,
    pub groupe_nom: Option<String>,
    pub count_total: u32,
    #[serde(default)]
    pub members: Vec<CosignPeer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Period {
    P30,
    P180,
    LEG,
}

impl Period {
    pub fn label(&self) -> &'static str {
        match self {
            Period::P30 => "30 jours",
            Period::P180 => "180 jours",
            Period::LEG => "Législature",
        }
    }

    pub fn json_file(&self) -> &'static str {
        match self {
            Period::P30 => "data/deputes_P30.json",
            Period::P180 => "data/deputes_P180.json",
            Period::LEG => "data/deputes_LEG.json",
        }
    }

    pub fn csv_file(&self) -> &'static str {
        match self {
            Period::P30 => "exports/deputes_activity_P30.csv",
            Period::P180 => "exports/deputes_activity_P180.csv",
            Period::LEG => "exports/deputes_activity_LEG.csv",
        }
    }
}

impl std::fmt::Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Nom,
    Groupe,
    Participation,
    AmdsAuthored,
    AmdsAdopted,
    AmdAdoptionRate,
    ScrutinsEligibles,
}

impl SortField {
    pub fn label(&self) -> &'static str {
        match self {
            SortField::Nom => "Nom",
            SortField::Groupe => "Groupe",
            SortField::Participation => "Participation",
            SortField::AmdsAuthored => "Amendements",
            SortField::AmdsAdopted => "Adoptés",
            SortField::AmdAdoptionRate => "Taux adoption",
            SortField::ScrutinsEligibles => "Scrutins éligibles",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

impl SortDir {
    pub fn toggle(self) -> Self {
        match self {
            SortDir::Asc => SortDir::Desc,
            SortDir::Desc => SortDir::Asc,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GroupPplIndex {
    pub version: u8,
    pub generated_at: String,
    pub total_groups: usize,
    pub total_ppl_links: usize,
    pub total_unique_ppl: usize,
    #[serde(default)]
    pub groups: Vec<GroupPplGroupIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GroupPplGroupIndexEntry {
    pub group_id: String,
    pub group_label: String,
    pub ppl_count: usize,
    pub authored_ppl_count: usize,
    pub cosigned_only_ppl_count: usize,
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GroupPplGroupShard {
    pub version: u8,
    pub generated_at: String,
    pub group_id: String,
    pub group_label: String,
    pub total_entries: usize,
    #[serde(default)]
    pub items: Vec<GroupPplItemSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GroupPplItemSummary {
    pub ppl_id: String,
    #[serde(default)]
    pub number: Option<String>,
    #[serde(default)]
    pub legislature: Option<u8>,
    pub title: String,
    #[serde(default)]
    pub deposit_date: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    pub author_count: u16,
    pub cosigner_count: u16,
    pub total_signers_from_group: u16,
    pub has_author: bool,
    #[serde(default)]
    pub signer_names_preview: Vec<String>,
    #[serde(default)]
    pub signers_preview: Vec<SignerPreviewEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SignerPreviewEntry {
    #[serde(default)]
    pub deputy_id: Option<String>,
    pub deputy_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeputyPplShard {
    pub version: u8,
    pub generated_at: String,
    pub deputy_id: String,
    pub deputy_name: String,
    #[serde(default)]
    pub group_id: Option<String>,
    #[serde(default)]
    pub group_label: Option<String>,
    pub total_entries: usize,
    pub authored_count: usize,
    pub cosigned_only_count: usize,
    #[serde(default)]
    pub items: Vec<DeputyPplItemSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeputyPplItemSummary {
    pub ppl_id: String,
    #[serde(default)]
    pub number: Option<String>,
    #[serde(default)]
    pub legislature: Option<u8>,
    pub title: String,
    #[serde(default)]
    pub deposit_date: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    pub is_author: bool,
    pub is_cosigner: bool,
}
