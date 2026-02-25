use anyhow::Result;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use std::collections::HashMap;
use std::time::Instant;
use tracing::info;

use crate::models::*;

pub struct AllAggregates {
    pub p30: Vec<DeputeStats>,
    pub p180: Vec<DeputeStats>,
    pub leg: Vec<DeputeStats>,
    pub deputes: Vec<Depute>,
    pub dossiers: HashMap<String, Dossier>,
}

// Début de la 17e législature
const LEG17_START: &str = "2022-06-19";

const COSIGN_TOP_LIMIT: usize = 10;
const COSIGN_IN_GROUP_UI_LIMIT: usize = 12;
const COSIGN_OUT_GROUP_MEMBERS_UI_LIMIT: usize = 8;

#[derive(Default)]
struct PeriodCosignAnalytics {
    top_by_dep: HashMap<String, Vec<TopCosignataire>>,
    network_by_dep: HashMap<String, CosignNetworkStats>,
}

#[derive(Debug, Clone, Copy)]
struct DateWindow {
    start: NaiveDate,
    end: NaiveDate,
}

fn merge_windows(mut windows: Vec<DateWindow>) -> Vec<DateWindow> {
    if windows.is_empty() {
        return windows;
    }
    windows.sort_by(|a, b| a.start.cmp(&b.start).then(a.end.cmp(&b.end)));

    let mut merged: Vec<DateWindow> = Vec::with_capacity(windows.len());
    for w in windows {
        if let Some(last) = merged.last_mut() {
            let contiguous_or_overlap = w.start <= last.end + Duration::days(1);
            if contiguous_or_overlap {
                if w.end > last.end {
                    last.end = w.end;
                }
                continue;
            }
        }
        merged.push(w);
    }
    merged
}

fn effective_mandate_windows(dep: &Depute, period_start: NaiveDate, period_end: NaiveDate) -> Vec<DateWindow> {
    let mut windows: Vec<DateWindow> = dep
        .mandat_assemblee_episodes
        .iter()
        .filter_map(|ep| {
            let start = ep.date_debut.max(period_start);
            let end = ep.date_fin.unwrap_or(period_end).min(period_end);
            (start <= end).then_some(DateWindow { start, end })
        })
        .collect();

    if windows.is_empty() {
        let start = dep.mandat_debut.unwrap_or(period_start).max(period_start);
        let end = dep.mandat_fin.unwrap_or(period_end).min(period_end);
        if start <= end {
            windows.push(DateWindow { start, end });
        }
    }

    merge_windows(windows)
}

fn date_in_windows(date: NaiveDate, windows: &[DateWindow]) -> bool {
    windows.iter().any(|w| date >= w.start && date <= w.end)
}

pub fn compute_all(raw: &RawDataset, now: DateTime<Utc>) -> Result<AllAggregates> {
    let today = now.date_naive();

    let p30_start  = today - Duration::days(30);
    let p180_start = today - Duration::days(180);
    let leg_start  = NaiveDate::parse_from_str(LEG17_START, "%Y-%m-%d").unwrap();

    let t_all = Instant::now();
    info!(
        "Agrégation détaillée: début (deputes={}, scrutins={}, amendements={})",
        raw.deputes.len(),
        raw.scrutins.len(),
        raw.amendements.len()
    );

    let t = Instant::now();
    let p30  = compute_period(raw, p30_start,  today, false);
    info!("Agrégation P30 OK en {:?} (lignes={})", t.elapsed(), p30.len());

    let t = Instant::now();
    let p180 = compute_period(raw, p180_start, today, false);
    info!("Agrégation P180 OK en {:?} (lignes={})", t.elapsed(), p180.len());

    // Pour la période LEG, on accepte les amendements sans date (déposés sur toute la législature)
    let t = Instant::now();
    let leg  = compute_period(raw, leg_start,  today, true);
    info!("Agrégation LEG OK en {:?} (lignes={})", t.elapsed(), leg.len());

    info!("Agrégation détaillée: terminée en {:?}", t_all.elapsed());

    Ok(AllAggregates {
        p30,
        p180,
        leg,
        deputes: raw.deputes.clone(),
        dossiers: raw.dossiers.clone(),
    })
}

fn compute_period(
    raw: &RawDataset,
    period_start: NaiveDate,
    period_end: NaiveDate,
    include_undated_amd: bool,
) -> Vec<DeputeStats> {
    let t_period = Instant::now();
    info!(
        "compute_period: début [{} -> {}] (deputes={}, scrutins={}, amendements={}, include_undated_amd={})",
        period_start,
        period_end,
        raw.deputes.len(),
        raw.scrutins.len(),
        raw.amendements.len(),
        include_undated_amd
    );

    // PERF: calcule le réseau de co-signatures une seule fois par période
    // (au lieu de rescanner tous les amendements pour chaque député).
    let t_cosign = Instant::now();
    let cosign_analytics =
        compute_cosign_analytics_for_period(raw, period_start, period_end, include_undated_amd);
    info!("compute_period: co-signatures prêtes en {:?}", t_cosign.elapsed());

    let total = raw.deputes.len();
    let out: Vec<DeputeStats> = raw.deputes
        .iter()
        .enumerate()
        .map(|(idx, dep)| {
            let done = idx + 1;
            if done == 1 || done % 100 == 0 || done == total {
                info!(
                    "compute_period: députés {}/{} [{} -> {}]",
                    done, total, period_start, period_end
                );
            }
            compute_depute_stats(
                dep,
                raw,
                period_start,
                period_end,
                include_undated_amd,
                &cosign_analytics,
            )
        })
        .collect();

    info!(
        "compute_period: terminé [{} -> {}] en {:?}",
        period_start,
        period_end,
        t_period.elapsed()
    );

    out
}

fn compute_depute_stats(
    dep: &Depute,
    raw: &RawDataset,
    period_start: NaiveDate,
    period_end: NaiveDate,
    include_undated_amd: bool,
    cosign_analytics: &PeriodCosignAnalytics,
) -> DeputeStats {
    let effective_windows = effective_mandate_windows(dep, period_start, period_end);

    let fallback_start = dep.mandat_debut.unwrap_or(period_start).max(period_start);
    let fallback_end = dep.mandat_fin.unwrap_or(period_end).min(period_end);

    let eff_start = effective_windows
        .first()
        .map(|w| w.start)
        .unwrap_or(fallback_start);
    let eff_end = effective_windows
        .last()
        .map(|w| w.end)
        .unwrap_or(fallback_end);

    // Guard: aucun épisode de mandat dans la période
    if effective_windows.is_empty() {
        return DeputeStats {
            deputy_id: dep.id.clone(),
            nom: dep.nom.clone(),
            prenom: dep.prenom.clone(),
            groupe_abrev: dep.groupe_abrev.clone(),
            groupe_nom: dep.groupe_nom.clone(),
            parti_rattachement: dep.parti_nom.clone(),
            dept: dep.dept_nom.clone(),
            circo: dep.circo.clone(),
            mandat_debut: dep.mandat_debut,
            mandat_fin: dep.mandat_fin,
            mandat_debut_legislature: dep.mandat_debut_legislature,
            mandat_assemblee_episodes: dep.mandat_assemblee_episodes.clone(),
            date_naissance: dep.date_naissance,
            sexe: dep.sexe.clone(),
            pays_naissance: dep.pays_naissance.clone(),
            profession: dep.profession.clone(),
            email_assemblee: dep.email_assemblee.clone(),
            site_web: dep.site_web.clone(),
            sites_web: dep.sites_web.clone(),
            sites_web_sources: dep.sites_web_sources.clone(),
            telephones: dep.telephones.clone(),
            uri_hatvp: dep.uri_hatvp.clone(),
            period_start: eff_start,
            period_end: eff_end,
            scrutins_eligibles: 0,
            votes_exprimes: 0,
            non_votant: 0,
            absent: 0,
            participation_rate: 0.0,
            pour_count: 0,
            contre_count: 0,
            abst_count: 0,
            amd_authored: 0,
            amd_adopted: 0,
            amd_adoption_rate: None,
            amd_cosigned: 0,
            interventions_count: 0,
            interventions_chars: 0,
            top_dossiers: vec![],
            top_cosignataires: vec![],
            cosign_network: None,
        };
    }

    // ── Scrutins ──────────────────────────────────────────────────────────────
    let mut scrutins_eligibles = 0u32;
    let mut votes_exprimes = 0u32;
    let mut non_votant = 0u32;
    let mut absent = 0u32;
    let mut pour_count = 0u32;
    let mut contre_count = 0u32;
    let mut abst_count = 0u32;
    let mut votes_par_dossier: HashMap<String, u32> = HashMap::new();

    for scrutin in &raw.scrutins {
        let date = match scrutin.date {
            Some(d) => d,
            None => continue, // scrutin sans date = non comptabilisable
        };
        if !date_in_windows(date, &effective_windows) {
            continue;
        }
        scrutins_eligibles += 1;

        match scrutin.votes.get(&dep.id) {
            Some(VotePosition::Pour) => {
                votes_exprimes += 1;
                pour_count += 1;
                if let Some(dref) = &scrutin.dossier_ref {
                    *votes_par_dossier.entry(dref.clone()).or_insert(0) += 1;
                }
            }
            Some(VotePosition::Contre) => {
                votes_exprimes += 1;
                contre_count += 1;
                if let Some(dref) = &scrutin.dossier_ref {
                    *votes_par_dossier.entry(dref.clone()).or_insert(0) += 1;
                }
            }
            Some(VotePosition::Abstention) => {
                votes_exprimes += 1;
                abst_count += 1;
                if let Some(dref) = &scrutin.dossier_ref {
                    *votes_par_dossier.entry(dref.clone()).or_insert(0) += 1;
                }
            }
            Some(VotePosition::NonVotant) => {
                non_votant += 1;
            }
            Some(VotePosition::Absent) | None => {
                absent += 1;
            }
        }
    }

    let participation_rate = if scrutins_eligibles > 0 {
        votes_exprimes as f64 / scrutins_eligibles as f64
    } else {
        0.0
    };

    // ── Amendements ──────────────────────────────────────────────────────────
    let mut amd_authored = 0u32;
    let mut amd_adopted  = 0u32;
    let mut amd_cosigned = 0u32;
    let mut amd_par_dossier: HashMap<String, u32> = HashMap::new();

    for amd in &raw.amendements {
        let in_window = match amd.date {
            Some(d) => date_in_windows(d, &effective_windows),
            // Sans date: inclure seulement pour LEG (on suppose toute la législature)
            None => include_undated_amd && !effective_windows.is_empty(),
        };
        if !in_window {
            continue;
        }

        if amd.auteur_id.as_deref() == Some(dep.id.as_str()) {
            amd_authored += 1;
            if amd.adopte {
                amd_adopted += 1;
            }
            if let Some(dref) = &amd.dossier_ref {
                *amd_par_dossier.entry(dref.clone()).or_insert(0) += 1;
            }
        } else if amd.cosignataires_ids.iter().any(|id| id == &dep.id) {
            amd_cosigned += 1;
        }
    }

    let amd_adoption_rate = if amd_authored > 0 {
        Some(amd_adopted as f64 / amd_authored as f64)
    } else {
        None
    };

    let top_cosignataires = cosign_analytics
        .top_by_dep
        .get(dep.id.as_str())
        .cloned()
        .unwrap_or_default();
    let cosign_network = cosign_analytics
        .network_by_dep
        .get(dep.id.as_str())
        .cloned();

    // ── Top dossiers ──────────────────────────────────────────────────────────
    let all_dossier_ids: std::collections::HashSet<&String> = votes_par_dossier.keys()
        .chain(amd_par_dossier.keys())
        .collect();

    let mut dossier_scores: Vec<DossierScore> = all_dossier_ids.iter()
        .filter_map(|did| {
            let v = votes_par_dossier.get(*did).copied().unwrap_or(0);
            let a = amd_par_dossier.get(*did).copied().unwrap_or(0);
            let score = v + 2 * a;
            if score == 0 { return None; }
            let titre = raw.dossiers.get(*did)
                .map(|d| d.titre.clone())
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| did.to_string());
            Some(DossierScore {
                dossier_id: did.to_string(),
                titre,
                votes: v,
                amendements: a,
                interventions: 0,
                score,
            })
        })
        .collect();

    dossier_scores.sort_by(|a, b| b.score.cmp(&a.score).then(a.dossier_id.cmp(&b.dossier_id)));
    dossier_scores.truncate(10);

    DeputeStats {
        deputy_id: dep.id.clone(),
        nom: dep.nom.clone(),
        prenom: dep.prenom.clone(),
        groupe_abrev: dep.groupe_abrev.clone(),
        groupe_nom: dep.groupe_nom.clone(),
        parti_rattachement: dep.parti_nom.clone(),
        dept: dep.dept_nom.clone(),
        circo: dep.circo.clone(),
        mandat_debut: dep.mandat_debut,
        mandat_fin: dep.mandat_fin,
        mandat_debut_legislature: dep.mandat_debut_legislature,
        mandat_assemblee_episodes: dep.mandat_assemblee_episodes.clone(),
        date_naissance: dep.date_naissance,
        sexe: dep.sexe.clone(),
        pays_naissance: dep.pays_naissance.clone(),
        profession: dep.profession.clone(),
        email_assemblee: dep.email_assemblee.clone(),
        site_web: dep.site_web.clone(),
        sites_web: dep.sites_web.clone(),
        sites_web_sources: dep.sites_web_sources.clone(),
        telephones: dep.telephones.clone(),
        uri_hatvp: dep.uri_hatvp.clone(),
        period_start: eff_start,
        period_end: eff_end,
        scrutins_eligibles,
        votes_exprimes,
        non_votant,
        absent,
        participation_rate,
        pour_count,
        contre_count,
        abst_count,
        amd_authored,
        amd_adopted,
        amd_adoption_rate,
        amd_cosigned,
        interventions_count: 0,
        interventions_chars: 0,
        top_dossiers: dossier_scores,
        top_cosignataires,
        cosign_network,
    }
}



fn compute_cosign_analytics_for_period(
    raw: &RawDataset,
    period_start: NaiveDate,
    period_end: NaiveDate,
    include_undated_amd: bool,
) -> PeriodCosignAnalytics {
    if raw.deputes.is_empty() || raw.amendements.is_empty() {
        return PeriodCosignAnalytics::default();
    }

    let dep_idx_by_id: HashMap<&str, usize> = raw
        .deputes
        .iter()
        .enumerate()
        .map(|(idx, d)| (d.id.as_str(), idx))
        .collect();

    let mut pair_counts: Vec<HashMap<usize, u32>> = vec![HashMap::new(); raw.deputes.len()];

    for amd in &raw.amendements {
        let in_window = match amd.date {
            Some(d) => d >= period_start && d <= period_end,
            None => include_undated_amd,
        };
        if !in_window {
            continue;
        }

        let mut signer_indices: Vec<usize> = Vec::with_capacity(1 + amd.cosignataires_ids.len());

        if let Some(author_id) = amd.auteur_id.as_deref() {
            if let Some(&idx) = dep_idx_by_id.get(author_id) {
                signer_indices.push(idx);
            }
        }
        for cid in &amd.cosignataires_ids {
            if let Some(&idx) = dep_idx_by_id.get(cid.as_str()) {
                signer_indices.push(idx);
            }
        }

        if signer_indices.len() < 2 {
            continue;
        }

        signer_indices.sort_unstable();
        signer_indices.dedup();
        if signer_indices.len() < 2 {
            continue;
        }

        for i in 0..signer_indices.len() {
            let a = signer_indices[i];
            for &b in &signer_indices[i + 1..] {
                *pair_counts[a].entry(b).or_insert(0) += 1;
                *pair_counts[b].entry(a).or_insert(0) += 1;
            }
        }
    }

    let mut top_by_dep: HashMap<String, Vec<TopCosignataire>> = HashMap::new();
    let mut network_by_dep: HashMap<String, CosignNetworkStats> = HashMap::new();

    for (dep_idx, dep) in raw.deputes.iter().enumerate() {
        let counts = &pair_counts[dep_idx];
        if counts.is_empty() {
            continue;
        }

        let mut pairs: Vec<(usize, u32)> = counts.iter().map(|(k, v)| (*k, *v)).collect();
        pairs.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then(raw.deputes[a.0].id.cmp(&raw.deputes[b.0].id))
        });

        let top = pairs
            .iter()
            .take(COSIGN_TOP_LIMIT)
            .map(|(other_idx, count)| {
                let other = &raw.deputes[*other_idx];
                TopCosignataire {
                    deputy_id: other.id.clone(),
                    nom: other.nom.clone(),
                    prenom: other.prenom.clone(),
                    groupe_abrev: other.groupe_abrev.clone(),
                    co_signed_count: *count,
                }
            })
            .collect::<Vec<_>>();
        if !top.is_empty() {
            top_by_dep.insert(dep.id.clone(), top);
        }

        let mut in_group_peers: Vec<CosignPeer> = Vec::new();
        let mut out_group_members: HashMap<(Option<String>, Option<String>), Vec<CosignPeer>> = HashMap::new();
        let mut in_group_count = 0u32;
        let mut out_group_count = 0u32;

        for (other_idx, count) in &pairs {
            let other = &raw.deputes[*other_idx];
            let same_group = dep.groupe_abrev.is_some()
                && other.groupe_abrev.is_some()
                && dep.groupe_abrev == other.groupe_abrev;

            let peer = CosignPeer {
                deputy_id: other.id.clone(),
                nom: other.nom.clone(),
                prenom: other.prenom.clone(),
                groupe_abrev: other.groupe_abrev.clone(),
                groupe_nom: other.groupe_nom.clone(),
                count: *count,
            };

            if same_group {
                in_group_count += *count;
                in_group_peers.push(peer);
            } else {
                out_group_count += *count;
                let key = (other.groupe_abrev.clone(), other.groupe_nom.clone());
                out_group_members.entry(key).or_default().push(peer);
            }
        }

        in_group_peers.sort_by(|a, b| {
            b.count
                .cmp(&a.count)
                .then(a.deputy_id.cmp(&b.deputy_id))
        });
        if in_group_peers.len() > COSIGN_IN_GROUP_UI_LIMIT {
            in_group_peers.truncate(COSIGN_IN_GROUP_UI_LIMIT);
        }

        let mut out_group_groups: Vec<CosignGroupBucket> = out_group_members
            .into_iter()
            .map(|((groupe_abrev, groupe_nom), mut members)| {
                members.sort_by(|a, b| {
                    b.count
                        .cmp(&a.count)
                        .then(a.deputy_id.cmp(&b.deputy_id))
                });
                let count_total: u32 = members.iter().map(|m| m.count).sum();
                if members.len() > COSIGN_OUT_GROUP_MEMBERS_UI_LIMIT {
                    members.truncate(COSIGN_OUT_GROUP_MEMBERS_UI_LIMIT);
                }
                CosignGroupBucket {
                    groupe_abrev,
                    groupe_nom,
                    count_total,
                    members,
                }
            })
            .collect();

        out_group_groups.sort_by(|a, b| {
            b.count_total
                .cmp(&a.count_total)
                .then(a.groupe_abrev.cmp(&b.groupe_abrev))
                .then(a.groupe_nom.cmp(&b.groupe_nom))
        });

        let total_cosignatures = in_group_count + out_group_count;
        let unique_cosignataires = counts.len() as u32;

        network_by_dep.insert(
            dep.id.clone(),
            CosignNetworkStats {
                total_cosignatures,
                unique_cosignataires,
                in_group_count,
                out_group_count,
                in_group: in_group_peers,
                out_group_groups,
            },
        );
    }

    PeriodCosignAnalytics { top_by_dep, network_by_dep }
}
