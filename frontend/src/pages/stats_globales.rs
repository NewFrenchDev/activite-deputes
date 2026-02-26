use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use leptos::*;
use leptos_router::A;

use crate::utils::app_href;
use crate::api::fetch_deputes;
use crate::models::{DeputeInfo, DeputeStats, Period};
use crate::store::use_store;

#[derive(Debug, Clone, Default)]
struct CountRow {
    label: String,
    count: usize,
}

#[derive(Debug, Clone, Default)]
struct HistogramBin {
    label: String,
    count: usize,
    from: f64,
    to: f64,
}

#[derive(Debug, Clone, Default)]
struct GroupActivityRow {
    group: String,
    seats: usize,
    median_participation_pct: Option<f64>,
    avg_participation_pct: Option<f64>,
    scrutins_ref: u32,
    amd_authored: u64,
    amd_adopted: u64,
}

#[derive(Debug, Clone)]
struct GlobalStatsPageSummary {
    period: Period,
    total_deputes: usize,
    total_groupes: usize,
    total_partis: usize,
    age_moyen: Option<f64>,
    age_median: Option<u32>,
    age_unknown: usize,
    femmes_pct: Option<f64>,
    participation_mediane_pct: Option<f64>,
    participation_moyenne_pct: Option<f64>,
    scrutins_reference: u32,
    amendements_deposes_total: u64,
    amendements_adoptes_total: u64,
    adoption_globale_pct: Option<f64>,
    professions_distinctes: usize,
    professions_non_renseignees: usize,
    sexes_non_renseignes: usize,
    groupes: Vec<CountRow>,
    partis: Vec<CountRow>,
    professions: Vec<CountRow>,
    sexes: Vec<CountRow>,
    ages: Vec<CountRow>,
    participation_histogram: Vec<HistogramBin>,
    group_activity: Vec<GroupActivityRow>,
}

#[derive(Debug, Default)]
struct GroupStatsAccumulator {
    seats: usize,
    participations: Vec<f64>,
    scrutins_ref: u32,
    amd_authored: u64,
    amd_adopted: u64,
}

#[component]
pub fn StatsGlobalesPage() -> impl IntoView {
    let store = use_store();
    let deputes_res = create_resource(|| (), |_| fetch_deputes());
    let (period, set_period) = create_signal(Period::P180);

    let store_for_header = store.clone();
    let store_for_page = store.clone();

    view! {
        <div class="reveal" style="display:flex;flex-direction:column;gap:1rem;">
            <section style="padding:1rem 1rem 1.1rem 1rem;border:1px solid var(--bg-border);border-radius:14px;background:linear-gradient(180deg, rgba(255,255,255,0.02), rgba(255,255,255,0));">
                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                    <div style="min-width:280px;flex:1;">
                        <div style="display:flex;align-items:center;gap:.45rem;flex-wrap:wrap;margin-bottom:.45rem;">
                            <span style="display:inline-flex;align-items:center;gap:.35rem;padding:.18rem .5rem;border-radius:999px;background:rgba(99,102,241,.14);color:var(--accent);font-size:.72rem;border:1px solid var(--accent-border);font-weight:600;">
                                "Analyse descriptive"
                            </span>
                            <span style="display:inline-flex;align-items:center;padding:.18rem .5rem;border-radius:999px;background:rgba(255,255,255,.03);color:var(--text-muted);font-size:.72rem;border:1px solid var(--bg-border);">
                                "17e législature"
                            </span>
                        </div>
                        <h1 style="margin:0 0 .35rem 0;font-size:1.35rem;font-weight:700;color:var(--text-primary);">
                            "Stats globales — Assemblée nationale"
                        </h1>
                        <p style="margin:0;color:var(--text-muted);line-height:1.45;font-size:.82rem;max-width:980px;">
                            "Vue d’ensemble factuelle de la composition des députés (groupe, parti, profession, sexe, âge) et d’indicateurs d’activité sur la période sélectionnée (participation, scrutins, amendements)."
                        </p>
                        {move || {
                            store_for_header.status.get().and_then(|r| r.ok()).map(|s| view! {
                                <p style="margin:.45rem 0 0 0;font-size:.74rem;color:var(--text-muted);">
                                    "Dernière mise à jour : "
                                    <strong style="color:var(--text-secondary);">{s.last_update_readable}</strong>
                                </p>
                            })
                        }}
                    </div>
                    <div style="display:flex;gap:.5rem;align-items:center;flex-wrap:wrap;">
                        <A href=app_href("/") class="btn">"← Retour accueil"</A>
                        <A href=app_href("/methodologie") class="btn" attr:style="text-decoration:none;">"Méthode & sources"</A>
                    </div>
                </div>

                <div style="display:flex;align-items:center;gap:.65rem;flex-wrap:wrap;margin-top:.85rem;">
                    <span style="font-size:.75rem;color:var(--text-muted);font-weight:600;">"Période d’activité"</span>
                    <div style="display:inline-flex;gap:.35rem;padding:.28rem;border-radius:10px;border:1px solid var(--bg-border);background:rgba(255,255,255,.02);">
                        {[Period::P30, Period::P180, Period::LEG]
                            .into_iter()
                            .map(|p| {
                                view! {
                                    <button
                                        on:click=move |_| set_period.set(p)
                                        style=move || {
                                            let active = period.get() == p;
                                            format!(
                                                "border:1px solid {};background:{};color:{};border-radius:8px;padding:.35rem .65rem;font-size:.76rem;font-weight:600;cursor:pointer;",
                                                if active { "var(--accent-border)" } else { "var(--bg-border)" },
                                                if active { "rgba(99,102,241,.14)" } else { "rgba(255,255,255,.02)" },
                                                if active { "var(--accent)" } else { "var(--text-secondary)" },
                                            )
                                        }
                                    >
                                        {p.label()}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>
                    <span style="font-size:.73rem;color:var(--text-muted);">
                        {move || format!("Lecture activité synchronisée avec la période : {}", period.get().label())}
                    </span>
                </div>
            </section>

            <div style="padding:.85rem .95rem;background:var(--accent-dim);border:1px solid var(--accent-border);border-radius:10px;font-size:.78rem;color:var(--text-secondary);line-height:1.45;">
                <strong style="color:var(--accent);">"Lecture"</strong>
                " : les cartes de composition (sexe, âge, professions, groupes, partis) sont calculées depuis "
                <code style="font-size:.75rem;color:var(--text-secondary);">"deputes.json"</code>
                " ; les cartes d’activité proviennent des datasets "
                <code style="font-size:.75rem;color:var(--text-secondary);">"deputes_P30/P180/LEG.json"</code>
                " selon la période sélectionnée. Les amendements correspondent aux indicateurs agrégés du site (déposés/adoptés/cosignés selon métriques existantes)."
            </div>

            {move || {
                let deputes_state = deputes_res.get();
                let stats_state = store_for_page.stats_for(period.get()).get();

                match (deputes_state, stats_state) {
                    (None, _) | (_, None) => view! { <StatsGlobalesSkeleton /> }.into_view(),
                    (Some(Err(e)), _) => view! {
                        <ErrorCard message=format!("Erreur chargement deputes.json : {e}") />
                    }.into_view(),
                    (_, Some(Err(e))) => view! {
                        <ErrorCard message=format!("Erreur chargement {} : {e}", period.get().json_file()) />
                    }.into_view(),
                    (Some(Ok(deputes)), Some(Ok(stats))) => {
                        let summary = compute_global_stats_page_summary(&deputes, &stats, period.get());
                        view! { <StatsGlobalesContent summary=summary /> }.into_view()
                    }
                }
            }}
        </div>
    }
}

#[component]
fn StatsGlobalesSkeleton() -> impl IntoView {
    view! {
        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:1rem;">
            {(0..8)
                .map(|_| {
                    view! {
                        <div style="height:94px;border-radius:12px;background:var(--bg-secondary);border:1px solid var(--bg-border);"></div>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
fn ErrorCard(message: String) -> impl IntoView {
    view! {
        <div style="padding:1rem;border:1px solid var(--danger);border-radius:10px;background:rgba(239,68,68,.08);color:var(--danger);">
            {message}
        </div>
    }
}

#[component]
fn StatsGlobalesContent(summary: GlobalStatsPageSummary) -> impl IntoView {
    let top_professions = top_n_with_other(summary.professions.clone(), 10);
    let top_partis = top_n_with_other(summary.partis.clone(), 12);

    let mut groups_by_median = summary.group_activity.clone();
    groups_by_median.sort_by(|a, b| {
        b.median_participation_pct
            .unwrap_or(-1.0)
            .total_cmp(&a.median_participation_pct.unwrap_or(-1.0))
            .then_with(|| b.seats.cmp(&a.seats))
    });

    let mut groups_by_amd = summary.group_activity.clone();
    groups_by_amd.sort_by(|a, b| {
        b.amd_authored
            .cmp(&a.amd_authored)
            .then_with(|| b.amd_adopted.cmp(&a.amd_adopted))
            .then_with(|| a.group.to_lowercase().cmp(&b.group.to_lowercase()))
    });
    let max_group_amd = groups_by_amd.iter().map(|g| g.amd_authored).max().unwrap_or(1);

    let mut group_adoption_rows: Vec<(String, usize, u64, u64, f64)> = summary
        .group_activity
        .iter()
        .filter_map(|g| {
            pct_opt(g.amd_adopted, g.amd_authored)
                .map(|rate| (g.group.clone(), g.seats, g.amd_authored, g.amd_adopted, rate))
        })
        .collect();
    group_adoption_rows.sort_by(|a, b| b.4.total_cmp(&a.4).then_with(|| a.0.cmp(&b.0)));
    let top_adoption_rate = group_adoption_rows.first().map(|x| x.4).unwrap_or(0.0);

    let sex_total = summary.total_deputes.max(1);
    let mut sex_rows = summary.sexes.clone();
    sex_rows.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.label.cmp(&b.label)));
    let femme_count = sex_rows
        .iter()
        .find(|r| r.label.to_lowercase().contains("femme"))
        .map(|r| r.count)
        .unwrap_or(0);
    let homme_count = sex_rows
        .iter()
        .find(|r| r.label.to_lowercase().contains("homme"))
        .map(|r| r.count)
        .unwrap_or(0);
    let sex_known = femme_count + homme_count;
    let femme_pct_total = if summary.total_deputes > 0 {
        femme_count as f64 * 100.0 / summary.total_deputes as f64
    } else {
        0.0
    };
    let femme_pct_known = if sex_known > 0 {
        femme_count as f64 * 100.0 / sex_known as f64
    } else {
        0.0
    };
    let homme_pct_known = if sex_known > 0 {
        homme_count as f64 * 100.0 / sex_known as f64
    } else {
        0.0
    };

    let sex_legend: Vec<(String, usize, f64, &'static str)> = sex_rows
        .iter()
        .map(|r| {
            let pct = if sex_total > 0 {
                r.count as f64 * 100.0 / sex_total as f64
            } else {
                0.0
            };
            (r.label.clone(), r.count, pct, sex_color(&r.label))
        })
        .collect();

    let donut_gradient = {
        let mut acc = 0.0f64;
        let mut segments: Vec<String> = Vec::new();
        for (_, count, _, color) in &sex_legend {
            if *count == 0 {
                continue;
            }
            let start = acc;
            let end = if sex_total > 0 {
                (acc + (*count as f64 * 100.0 / sex_total as f64)).min(100.0)
            } else {
                acc
            };
            segments.push(format!("{color} {start:.3}% {end:.3}%"));
            acc = end;
        }
        if segments.is_empty() {
            "conic-gradient(rgba(148,163,184,.55) 0% 100%)".to_string()
        } else {
            format!("conic-gradient({})", segments.join(", "))
        }
    };

    let age_known_total = summary.total_deputes.saturating_sub(summary.age_unknown).max(1);
    let age_max = summary.ages.iter().map(|r| r.count).max().unwrap_or(1);
    let prof_max = top_professions.iter().map(|r| r.count).max().unwrap_or(1);
    let seats_max = summary.groupes.iter().map(|r| r.count).max().unwrap_or(1);
    let hist_max = summary
        .participation_histogram
        .iter()
        .map(|b| b.count)
        .max()
        .unwrap_or(1);

    view! {
        <div class="sg-wrap">
            <style>{r#"
                .sg-wrap{position:relative;overflow:hidden;padding:1.1rem;border-radius:16px;border:1px solid var(--bg-border);background:linear-gradient(180deg, rgba(8,9,14,.92), rgba(10,12,18,.94));}
                .sg-bg-grid,.sg-bg-glow{position:absolute;pointer-events:none;z-index:0;}
                .sg-bg-grid{inset:0;opacity:.32;background-image:radial-gradient(circle, rgba(30,37,53,.95) 1px, transparent 1px);background-size:28px 28px;animation:sg-grid-drift 60s linear infinite;}
                .sg-bg-glow{top:-70px;left:26%;right:26%;height:200px;background:radial-gradient(ellipse, rgba(34,211,238,.12) 0%, transparent 70%);animation:sg-glow-pulse 7s ease-in-out infinite;}
                .sg-body{position:relative;z-index:1;display:flex;flex-direction:column;gap:1rem;}
                .sg-kpis{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:.65rem;}
                .sg-kpi{background:rgba(15,17,23,.88);border:1px solid rgba(30,37,53,.9);border-radius:12px;padding:.9rem 1rem;position:relative;overflow:hidden;animation:sg-pop .55s cubic-bezier(.22,1,.36,1) both;transition:border-color .18s ease, transform .18s ease;}
                .sg-kpi:hover{border-color:rgba(44,54,80,.95);transform:translateY(-2px);}
                .sg-kpi::after{content:"";position:absolute;top:0;left:-100%;right:100%;height:2px;background:linear-gradient(90deg, transparent, rgba(34,211,238,.95), transparent);transition:left .35s ease,right .35s ease;}
                .sg-kpi:hover::after{left:0;right:0;}
                .sg-kpi-val{font-size:1.35rem;line-height:1;font-weight:800;letter-spacing:-.03em;color:var(--text-primary);font-variant-numeric:tabular-nums;}
                .sg-kpi-val.accent{color:#22d3ee;}
                .sg-kpi-label{margin-top:.35rem;font-size:.68rem;color:var(--text-muted);font-weight:600;letter-spacing:.04em;text-transform:uppercase;}
                .sg-kpi-sub{margin-top:.15rem;font-size:.68rem;color:#7f8ca7;line-height:1.35;}
                .sg-note{padding:.7rem .9rem;border-radius:9px;background:rgba(34,211,238,.06);border:1px solid rgba(34,211,238,.16);color:#93a5c2;font-size:.76rem;line-height:1.5;animation:sg-fade-up .6s .08s cubic-bezier(.22,1,.36,1) both;}
                .sg-note strong{color:#22d3ee;}
                .sg-section-title{font-size:.66rem;font-weight:700;letter-spacing:.12em;text-transform:uppercase;color:#64748b;display:flex;align-items:center;gap:.6rem;margin-top:.15rem;animation:sg-fade-left .45s cubic-bezier(.22,1,.36,1) both;}
                .sg-section-title::after{content:"";height:1px;flex:1;background:rgba(30,37,53,.9);}
                .sg-grid-2{display:grid;grid-template-columns:repeat(auto-fit,minmax(420px,1fr));gap:1rem;}
                .sg-grid-3{display:grid;grid-template-columns:repeat(auto-fit,minmax(290px,1fr));gap:1rem;}
                .sg-card{background:rgba(15,17,23,.90);border:1px solid rgba(30,37,53,.92);border-radius:14px;padding:1rem 1.05rem;animation:sg-fade-up .55s cubic-bezier(.22,1,.36,1) both;}
                .sg-card:hover{border-color:rgba(44,54,80,.95);}
                .sg-card-title{font-size:.82rem;font-weight:700;color:var(--text-primary);margin:0 0 .2rem 0;}
                .sg-card-sub{font-size:.72rem;color:#8896b3;line-height:1.45;margin:0 0 .95rem 0;}
                .sg-legend-inline{display:flex;gap:.9rem;flex-wrap:wrap;font-size:.68rem;color:#92a0bb;}
                .sg-dot{width:10px;height:10px;border-radius:999px;display:inline-block;flex:none;border:1px solid rgba(255,255,255,.12);}
                .sg-hist-wrap{position:relative;}
                .sg-hist-grid{display:grid;grid-template-columns:repeat(10,minmax(0,1fr));gap:4px;align-items:end;height:128px;}
                .sg-hcol{display:flex;flex-direction:column;align-items:center;gap:4px;min-width:0;}
                .sg-hcount{font-size:.62rem;color:#8fa0bf;min-height:12px;font-variant-numeric:tabular-nums;}
                .sg-hbar-track{height:96px;width:100%;display:flex;align-items:flex-end;}
                .sg-hbar{width:100%;height:0;border-radius:4px 4px 0 0;border:1px solid rgba(99,102,241,.22);background:linear-gradient(180deg, rgba(99,102,241,.85), rgba(99,102,241,.30));animation:sg-grow-h .75s cubic-bezier(.22,1,.36,1) forwards;}
                .sg-hbar.dim{border-color:rgba(55,65,81,.35);background:linear-gradient(180deg, rgba(55,65,81,.75), rgba(55,65,81,.20));}
                .sg-hlbl{font-size:.58rem;color:#667892;font-family:monospace;}
                .sg-med-line{position:absolute;top:14px;bottom:18px;width:2px;border-radius:2px;background:#fbbf24;opacity:.85;animation:sg-fade-in .5s .6s both;}
                .sg-med-tag{position:absolute;top:0;transform:translateX(-50%);color:#fbbf24;font-size:.58rem;font-family:monospace;white-space:nowrap;animation:sg-fade-in .5s .8s both;}
                .sg-bar-list{display:flex;flex-direction:column;gap:.55rem;}
                .sg-row3{display:grid;grid-template-columns:66px 1fr auto;gap:.6rem;align-items:center;}
                .sg-row3-wide{display:grid;grid-template-columns:minmax(68px,88px) 1fr 74px;gap:.55rem;align-items:center;}
                .sg-code{font-size:.71rem;font-weight:700;font-family:monospace;}
                .sg-track{height:8px;background:rgba(22,27,38,.92);border-radius:3px;overflow:hidden;border:1px solid rgba(30,37,53,.9);}
                .sg-track.tall{height:12px;position:relative;}
                .sg-fill{height:100%;width:0;border-radius:2px;animation:sg-grow-w .85s cubic-bezier(.22,1,.36,1) forwards;}
                .sg-fill.soft{opacity:.23;position:absolute;left:0;top:0;}
                .sg-fill-main{opacity:.82;position:absolute;left:0;top:0;}
                .sg-right-num{font-size:.72rem;color:#93a3c2;text-align:right;font-family:monospace;font-variant-numeric:tabular-nums;}
                .sg-split-legend{display:flex;gap:.9rem;flex-wrap:wrap;font-size:.68rem;color:#93a3c2;margin-bottom:.55rem;}
                .sg-pill{display:inline-flex;align-items:center;gap:.35rem;}
                .sg-donut-wrap{display:flex;align-items:center;gap:1rem;flex-wrap:wrap;}
                .sg-donut{width:102px;height:102px;border-radius:999px;position:relative;flex:none;border:1px solid rgba(30,37,53,.9);animation:sg-pop .55s cubic-bezier(.22,1,.36,1) both;}
                .sg-donut::after{content:"";position:absolute;inset:18px;border-radius:999px;background:rgba(15,17,23,.97);border:1px solid rgba(30,37,53,.9);box-shadow: inset 0 0 0 1px rgba(255,255,255,.02);}
                .sg-donut-center{position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;z-index:1;text-align:center;pointer-events:none;}
                .sg-donut-center .v{font-size:.95rem;font-weight:800;color:#22d3ee;line-height:1;}
                .sg-donut-center .l{font-size:.57rem;color:#8fa0bf;font-family:monospace;margin-top:.15rem;}
                .sg-donut-legend{display:flex;flex-direction:column;gap:.45rem;min-width:180px;flex:1;}
                .sg-donut-row{display:grid;grid-template-columns:auto 1fr auto;gap:.5rem;align-items:center;}
                .sg-donut-name{font-size:.75rem;color:var(--text-secondary);}
                .sg-donut-val{font-size:.72rem;color:#93a3c2;font-family:monospace;}
                .sg-gender-bar{height:8px;border-radius:4px;background:rgba(22,27,38,.92);overflow:hidden;border:1px solid rgba(30,37,53,.9);margin-top:.8rem;}
                .sg-gender-fill{height:100%;width:0;border-radius:4px;background:linear-gradient(90deg,#a78bfa,#7c3aed);animation:sg-grow-w .9s .12s cubic-bezier(.22,1,.36,1) forwards;}
                .sg-gender-meta{display:flex;justify-content:space-between;margin-top:.25rem;font-size:.62rem;color:#64748b;font-family:monospace;gap:.5rem;flex-wrap:wrap;}
                .sg-age-list,.sg-prof-list{display:flex;flex-direction:column;gap:.45rem;}
                .sg-age-row{display:grid;grid-template-columns:74px 1fr 58px;gap:.55rem;align-items:center;}
                .sg-age-l{font-size:.71rem;color:#90a0bf;font-family:monospace;}
                .sg-age-track{height:16px;background:rgba(22,27,38,.92);border-radius:3px;overflow:hidden;border:1px solid rgba(30,37,53,.9);}
                .sg-age-fill{height:100%;width:0;border-radius:3px;background:linear-gradient(90deg,#7c3aed,#6366f1);animation:sg-grow-w .9s cubic-bezier(.22,1,.36,1) forwards;}
                .sg-age-r{font-size:.69rem;color:#93a3c2;text-align:right;font-family:monospace;}
                .sg-age-foot{display:flex;justify-content:space-between;gap:.75rem;flex-wrap:wrap;margin-top:.7rem;padding-top:.7rem;border-top:1px solid rgba(30,37,53,.9);font-size:.69rem;color:#8fa0bf;font-family:monospace;}
                .sg-age-foot strong{color:var(--text-primary);}
                .sg-prof-row{display:grid;grid-template-columns:minmax(0,1fr) 118px 54px;gap:.55rem;align-items:center;}
                .sg-prof-name{font-size:.72rem;color:#8ea0bf;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;}
                .sg-prof-track{height:6px;background:rgba(22,27,38,.92);border-radius:2px;overflow:hidden;border:1px solid rgba(30,37,53,.9);}
                .sg-prof-fill{height:100%;width:0;border-radius:2px;background:rgba(34,211,238,.75);animation:sg-grow-w .8s cubic-bezier(.22,1,.36,1) forwards;}
                .sg-prof-c{font-size:.69rem;color:#93a3c2;text-align:right;font-family:monospace;}
                .sg-seats-strip{height:18px;border-radius:5px;overflow:hidden;display:flex;gap:1px;background:rgba(22,27,38,.92);padding:1px;border:1px solid rgba(30,37,53,.9);}
                .sg-seat-seg{height:100%;flex:0 0 0%;opacity:.78;transition:opacity .18s;animation:sg-flex-grow .85s cubic-bezier(.22,1,.36,1) forwards;}
                .sg-seat-seg:hover{opacity:1;}
                .sg-seat-list{display:grid;grid-template-columns:repeat(auto-fit,minmax(230px,1fr));gap:.55rem;margin-top:.9rem;}
                .sg-seat-item{display:grid;grid-template-columns:auto 1fr auto;gap:.5rem;align-items:center;}
                .sg-seat-label{font-size:.73rem;color:var(--text-secondary);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;}
                .sg-seat-num{font-size:.7rem;color:#93a3c2;font-family:monospace;}
                .sg-subtle{font-size:.72rem;color:#8ea0bf;line-height:1.45;}
                @media (max-width: 980px){.sg-grid-2{grid-template-columns:1fr;}.sg-row3{grid-template-columns:58px 1fr 68px;}.sg-row3-wide{grid-template-columns:58px 1fr 70px;}}
                @media (max-width: 640px){.sg-wrap{padding:.8rem;}.sg-kpis{grid-template-columns:repeat(auto-fit,minmax(132px,1fr));}.sg-grid-3{grid-template-columns:1fr;}.sg-donut-wrap{gap:.8rem;}.sg-donut-legend{min-width:0;width:100%;}.sg-seat-list{grid-template-columns:1fr;}.sg-prof-row{grid-template-columns:minmax(0,1fr) 90px 46px;}}
                @keyframes sg-grow-w { from{width:0;} to{width:var(--w);} }
                @keyframes sg-grow-h { from{height:0;} to{height:var(--h);} }
                @keyframes sg-flex-grow { from{flex-basis:0%;} to{flex-basis:var(--f);} }
                @keyframes sg-fade-up { from{opacity:0;transform:translateY(18px);} to{opacity:1;transform:translateY(0);} }
                @keyframes sg-fade-left { from{opacity:0;transform:translateX(-12px);} to{opacity:1;transform:translateX(0);} }
                @keyframes sg-fade-in { from{opacity:0;} to{opacity:1;} }
                @keyframes sg-pop { from{opacity:0;transform:translateY(14px) scale(.97);} to{opacity:1;transform:translateY(0) scale(1);} }
                @keyframes sg-grid-drift { from{background-position:0 0;} to{background-position:28px 28px;} }
                @keyframes sg-glow-pulse { 0%,100%{opacity:.65;transform:scaleX(1);} 50%{opacity:1;transform:scaleX(1.15);} }
            "#}</style>

            <div class="sg-bg-grid"></div>
            <div class="sg-bg-glow"></div>

            <div class="sg-body">
                <div class="sg-kpis">
                    <SGMetricTile label="Députés" value={fmt_int(summary.total_deputes as u64)} sub={"mandat actif".to_string()} accent=false delay_ms=0 />
                    <SGMetricTile label="Participation médiane" value={summary.participation_mediane_pct.map(fmt_pct1).unwrap_or_else(|| "—".to_string())} sub={format!("fenêtre {}", summary.period.label())} accent=true delay_ms=60 />
                    <SGMetricTile label="Âge médian" value={summary.age_median.map(|v| format!("{v} ans")).unwrap_or_else(|| "—".to_string())} sub={summary.age_moyen.map(|v| format!("moy. {}", fmt_age1(v))).unwrap_or_else(|| "âge moyen indisponible".to_string())} accent=false delay_ms=120 />
                    <SGMetricTile label="Femmes élues" value={format!("{:.1}%", femme_pct_total)} sub={format!("{} / {}", fmt_int(femme_count as u64), fmt_int(summary.total_deputes as u64))} accent=false delay_ms=180 />
                    <SGMetricTile label="Amendements déposés" value={fmt_int(summary.amendements_deposes_total)} sub={format!("sur {}", summary.period.label())} accent=false delay_ms=240 />
                    <SGMetricTile label="Adoptés" value={fmt_int(summary.amendements_adoptes_total)} sub={summary.adoption_globale_pct.map(|v| format!("taux global {}", fmt_pct1(v))).unwrap_or_else(|| "taux global —".to_string())} accent=false delay_ms=300 />
                    <SGMetricTile label="Scrutins publics" value={if summary.scrutins_reference > 0 { fmt_int(summary.scrutins_reference as u64) } else { "—".to_string() }} sub={format!("fenêtre {}", summary.period.label())} accent=false delay_ms=360 />
                    <SGMetricTile label="Groupes / partis" value={format!("{} / {}", summary.total_groupes, summary.total_partis)} sub={"composition politique".to_string()} accent=false delay_ms=420 />
                </div>

                <div class="sg-note">
                    <strong>"Participation"</strong>
                    " = votes exprimés (Pour / Contre / Abstention) sur scrutins publics, pas la présence physique en hémicycle. Amendements : auteur principal uniquement dans les agrégats de ce site."
                </div>

                <SGSectionTitle title="Distribution de la participation" delay_ms=0 />

                <div class="sg-grid-2">
                    <SGCard title="Distribution par tranche" subtitle={format!("Nombre de députés par tranche de 10 points ({}) · médiane {}.", summary.period.label(), summary.participation_mediane_pct.map(fmt_pct1).unwrap_or_else(|| "—".to_string()))} delay_ms=0>
                        <div class="sg-hist-wrap">
                            <div class="sg-hist-grid">
                                {summary.participation_histogram.iter().enumerate().map(|(i, b)| {
                                    let h = if hist_max > 0 { (b.count as f64 * 96.0 / hist_max as f64).clamp(0.0, 96.0) } else { 0.0 };
                                    let is_main = i >= 3;
                                    view! {
                                        <div class="sg-hcol" title=format!("{} : {} député(s)", b.label, b.count)>
                                            <div class="sg-hcount">{if b.count > 0 { b.count.to_string() } else { "".to_string() }}</div>
                                            <div class="sg-hbar-track">
                                                <div class=if is_main { "sg-hbar" } else { "sg-hbar dim" } style=format!("--h:{h:.2}px;animation-delay:{}ms;", 120 + i * 45)></div>
                                            </div>
                                            <div class="sg-hlbl">{b.label.replace('%', "")}</div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                            {summary.participation_mediane_pct.map(|med| {
                                let left = med.clamp(0.0, 100.0);
                                view! {
                                    <>
                                        <div class="sg-med-line" style=format!("left:{left:.2}%;")></div>
                                        <div class="sg-med-tag" style=format!("left:{left:.2}%;")>{format!("▲ {}", fmt_pct1(med))}</div>
                                    </>
                                }
                            })}
                        </div>
                        <div class="sg-legend-inline" style="margin-top:.7rem;">
                            <span class="sg-pill"><span class="sg-dot" style="background:rgba(99,102,241,.75);border:none;"></span>"Nb députés / tranche"</span>
                            <span class="sg-pill"><span class="sg-dot" style="width:10px;height:3px;border-radius:2px;background:#fbbf24;border:none;"></span>"Médiane"</span>
                        </div>
                    </SGCard>

                    <SGCard title="Participation médiane par groupe" subtitle={"Médiane des taux individuels au sein de chaque groupe — tri décroissant.".to_string()} delay_ms=70>
                        <div class="sg-bar-list">
                            {groups_by_median.iter().take(12).enumerate().map(|(i, g)| {
                                let rate = g.median_participation_pct.unwrap_or(0.0).clamp(0.0, 100.0);
                                let color = group_palette_color(&g.group);
                                view! {
                                    <div class="sg-row3">
                                        <div class="sg-code" style=format!("color:{color};")>{g.group.clone()}</div>
                                        <div class="sg-track"><div class="sg-fill" style=format!("--w:{rate:.2}%;background:{color};opacity:.75;animation-delay:{}ms;", 100 + i * 45)></div></div>
                                        <div class="sg-right-num" style=format!("color:{color};")>{fmt_pct1(rate)}</div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </SGCard>
                </div>

                <SGSectionTitle title="Activité législative — amendements" delay_ms=40 />

                <div class="sg-grid-2">
                    <SGCard title="Amendements déposés vs adoptés par groupe" subtitle={"Couleur atténuée = déposés · couleur pleine = adoptés. Volumes comparés sur une échelle commune.".to_string()} delay_ms=100>
                        <div class="sg-split-legend">
                            <span class="sg-pill"><span class="sg-dot" style="background:rgba(99,102,241,.25);border:none;"></span>"Déposés"</span>
                            <span class="sg-pill"><span class="sg-dot" style="background:rgba(99,102,241,.85);border:none;"></span>"Adoptés"</span>
                        </div>
                        <div class="sg-bar-list">
                            {groups_by_amd.iter().take(12).enumerate().map(|(i, g)| {
                                let color = group_palette_color(&g.group);
                                let dep_pct = if max_group_amd > 0 { (g.amd_authored as f64 * 100.0 / max_group_amd as f64).clamp(0.0, 100.0) } else { 0.0 };
                                let ado_pct = if max_group_amd > 0 { (g.amd_adopted as f64 * 100.0 / max_group_amd as f64).clamp(0.0, 100.0) } else { 0.0 };
                                view! {
                                    <div class="sg-row3-wide" title=format!("{} · déposés {} · adoptés {}", g.group, fmt_int(g.amd_authored), fmt_int(g.amd_adopted))>
                                        <div class="sg-code" style=format!("color:{color};")>{g.group.clone()}</div>
                                        <div class="sg-track tall">
                                            <div class="sg-fill soft" style=format!("--w:{dep_pct:.2}%;background:{color};animation-delay:{}ms;", 90 + i * 35)></div>
                                            <div class="sg-fill sg-fill-main" style=format!("--w:{ado_pct:.2}%;background:{color};animation-delay:{}ms;", 150 + i * 35)></div>
                                        </div>
                                        <div class="sg-right-num">{fmt_int(g.amd_authored)}</div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </SGCard>

                    <SGCard title="Taux d'adoption par groupe" subtitle={"Adoptés / déposés. Vert >15%, orange 8–15%, rouge <8%.".to_string()} delay_ms=170>
                        <div class="sg-bar-list">
                            {group_adoption_rows.iter().take(12).enumerate().map(|(i, (label, _seats, dep, ado, rate))| {
                                let rate_color = if *rate > 15.0 { "rgba(52,211,153,.88)" } else if *rate > 8.0 { "rgba(251,191,36,.88)" } else { "rgba(248,113,113,.88)" };
                                let w = if top_adoption_rate > 0.0 { (*rate * 100.0 / top_adoption_rate).clamp(0.0, 100.0) } else { 0.0 };
                                view! {
                                    <div class="sg-row3" title=format!("{} · {}/{} = {}", label, fmt_int(*ado), fmt_int(*dep), fmt_pct1(*rate))>
                                        <div class="sg-code" style=format!("color:{};", group_palette_color(label))>{label.clone()}</div>
                                        <div class="sg-track"><div class="sg-fill" style=format!("--w:{w:.2}%;background:{rate_color};animation-delay:{}ms;", 100 + i * 45)></div></div>
                                        <div class="sg-right-num" style=format!("color:{rate_color};")>{fmt_pct1(*rate)}</div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </SGCard>
                </div>

                <SGSectionTitle title="Composition de l'Assemblée" delay_ms=80 />

                <div class="sg-grid-3">
                    <SGCard title="Répartition femmes / hommes" subtitle={"Civilité déclarée dans le dataset officiel (normalisée).".to_string()} delay_ms=220>
                        <div class="sg-donut-wrap">
                            <div class="sg-donut" style=format!("background:{};", donut_gradient)>
                                <div class="sg-donut-center">
                                    <div class="v">{format!("{:.1}%", femme_pct_total)}</div>
                                    <div class="l">"Femmes / total"</div>
                                </div>
                            </div>
                            <div class="sg-donut-legend">
                                {sex_legend.iter().enumerate().map(|(i, (label, count, pct, color))| {
                                    view! {
                                        <div class="sg-donut-row" style=format!("animation: sg-fade-up .45s {}ms both;", 160 + i * 50)>
                                            <span class="sg-dot" style=format!("background:{};border:none;", color)></span>
                                            <span class="sg-donut-name">{label.clone()}</span>
                                            <span class="sg-donut-val">{format!("{} · {}", fmt_int(*count as u64), fmt_pct1(*pct))}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                        <div class="sg-gender-bar"><div class="sg-gender-fill" style=format!("--w:{:.2}%;", femme_pct_known.clamp(0.0, 100.0))></div></div>
                        <div class="sg-gender-meta">
                            <span>{format!("Femmes (connues) {}", fmt_pct1(femme_pct_known))}</span>
                            <span>{format!("Hommes (connus) {}", fmt_pct1(homme_pct_known))}</span>
                        </div>
                    </SGCard>

                    <SGCard title="Répartition par tranche d'âge" subtitle={"Calculée depuis la date de naissance déclarée.".to_string()} delay_ms=290>
                        <div class="sg-age-list">
                            {summary.ages.iter().enumerate().map(|(i, row)| {
                                let w = if age_max > 0 { (row.count as f64 * 100.0 / age_max as f64).clamp(0.0, 100.0) } else { 0.0 };
                                let pct = if age_known_total > 0 { row.count as f64 * 100.0 / age_known_total as f64 } else { 0.0 };
                                view! {
                                    <div class="sg-age-row" title=format!("{} : {} ({})", row.label, row.count, fmt_pct1(pct))>
                                        <div class="sg-age-l">{row.label.clone()}</div>
                                        <div class="sg-age-track"><div class="sg-age-fill" style=format!("--w:{w:.2}%;animation-delay:{}ms;", 100 + i * 55)></div></div>
                                        <div class="sg-age-r">{row.count}</div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                        <div class="sg-age-foot">
                            <span>"Médiane : " <strong>{summary.age_median.map(|v| format!("{v} ans")).unwrap_or_else(|| "—".to_string())}</strong></span>
                            <span>"Moyenne : " <strong>{summary.age_moyen.map(fmt_age1).unwrap_or_else(|| "—".to_string())}</strong></span>
                        </div>
                        {if summary.age_unknown > 0 {
                            view! { <div class="sg-subtle" style="margin-top:.55rem;">{format!("{} profil(s) sans date de naissance exploitable.", summary.age_unknown)}</div> }.into_view()
                        } else {
                            view! { <></> }.into_view()
                        }}
                    </SGCard>

                    <SGCard title="Professions les plus représentées" subtitle={"Libellés normalisés (suppression des préfixes codés type “(33) -”).".to_string()} delay_ms=360>
                        <div class="sg-prof-list">
                            {top_professions.iter().enumerate().map(|(i, row)| {
                                let w = if prof_max > 0 { (row.count as f64 * 100.0 / prof_max as f64).clamp(0.0, 100.0) } else { 0.0 };
                                view! {
                                    <div class="sg-prof-row" title=format!("{} : {}", row.label, row.count)>
                                        <div class="sg-prof-name">{row.label.clone()}</div>
                                        <div class="sg-prof-track"><div class="sg-prof-fill" style=format!("--w:{w:.2}%;animation-delay:{}ms;", 110 + i * 45)></div></div>
                                        <div class="sg-prof-c">{row.count}</div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                        <div class="sg-subtle" style="margin-top:.7rem;">
                            {format!("{} profession(s) distincte(s) • {} non renseigné(e)(s)", summary.professions_distinctes, summary.professions_non_renseignees)}
                        </div>
                    </SGCard>
                </div>

                <SGSectionTitle title="Composition politique" delay_ms=120 />

                <SGCard title="Répartition des sièges par groupe" subtitle={"Proportions sur les sièges observés · détail + bandeau synthétique.".to_string()} delay_ms=430>
                    <div class="sg-seats-strip">
                        {summary.groupes.iter().enumerate().map(|(i, row)| {
                            let color = group_palette_color(&row.label);
                            let pct = if summary.total_deputes > 0 { row.count as f64 * 100.0 / summary.total_deputes as f64 } else { 0.0 };
                            view! { <div class="sg-seat-seg" title=format!("{} : {} ({})", row.label, row.count, fmt_pct1(pct)) style=format!("--f:{:.4}%;background:{};animation-delay:{}ms;", pct.clamp(0.0, 100.0), color, 80 + i * 30)></div> }
                        }).collect_view()}
                    </div>
                    <div class="sg-seat-list">
                        {summary.groupes.iter().enumerate().map(|(i, row)| {
                            let color = group_palette_color(&row.label);
                            let w = if seats_max > 0 { (row.count as f64 * 100.0 / seats_max as f64).clamp(0.0, 100.0) } else { 0.0 };
                            let pct = if summary.total_deputes > 0 { row.count as f64 * 100.0 / summary.total_deputes as f64 } else { 0.0 };
                            view! {
                                <div class="sg-seat-item" title=format!("{} : {} ({})", row.label, row.count, fmt_pct1(pct))>
                                    <span class="sg-dot" style=format!("background:{};border:none;", color)></span>
                                    <div>
                                        <div class="sg-seat-label">{row.label.clone()}</div>
                                        <div class="sg-track" style="height:6px;margin-top:.22rem;"><div class="sg-fill" style=format!("--w:{w:.2}%;background:{color};opacity:.78;animation-delay:{}ms;", 120 + i * 35)></div></div>
                                    </div>
                                    <div class="sg-seat-num">{format!("{} · {}", row.count, fmt_pct1(pct))}</div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </SGCard>

                <div class="sg-grid-2">
                    <SGCard title="Partis de rattachement" subtitle={"Répartition descriptive des partis déclarés (top 12 + autres).".to_string()} delay_ms=500>
                        <div class="sg-prof-list">
                            {top_partis.iter().enumerate().map(|(i, row)| {
                                let maxp = top_partis.first().map(|x| x.count).unwrap_or(1);
                                let w = if maxp > 0 { (row.count as f64 * 100.0 / maxp as f64).clamp(0.0, 100.0) } else { 0.0 };
                                view! {
                                    <div class="sg-prof-row" title=format!("{} : {}", row.label, row.count)>
                                        <div class="sg-prof-name">{row.label.clone()}</div>
                                        <div class="sg-prof-track"><div class="sg-prof-fill" style=format!("--w:{w:.2}%;animation-delay:{}ms;background:rgba(99,102,241,.70);", 100 + i * 35)></div></div>
                                        <div class="sg-prof-c">{row.count}</div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </SGCard>

                    <SGCard title="Repères de lecture" subtitle={"Aide à l’interprétation des indicateurs affichés.".to_string()} delay_ms=560>
                        <div class="sg-subtle" style="display:flex;flex-direction:column;gap:.55rem;">
                            <div><strong style="color:var(--text-primary);">"Participation"</strong>" : proportion de scrutins publics avec vote nominal observé pour un député."</div>
                            <div><strong style="color:var(--text-primary);">"Médiane par groupe"</strong>" : comparaison robuste aux valeurs extrêmes."</div>
                            <div><strong style="color:var(--text-primary);">"Taux d’adoption"</strong>" : à lire avec le volume de dépôts."</div>
                            <div><strong style="color:var(--text-primary);">"Fenêtre"</strong>{format!(" : tous les indicateurs d’activité suivent la période {}.", summary.period.label())}</div>
                        </div>
                    </SGCard>
                </div>
            </div>
        </div>
    }
}

#[component]
fn SGMetricTile(
    label: &'static str,
    value: String,
    sub: String,
    accent: bool,
    delay_ms: usize,
) -> impl IntoView {
    view! {
        <div class="sg-kpi" style=format!("animation-delay:{}ms;", delay_ms)>
            <div class=if accent { "sg-kpi-val accent" } else { "sg-kpi-val" }>{value}</div>
            <div class="sg-kpi-label">{label}</div>
            <div class="sg-kpi-sub">{sub}</div>
        </div>
    }
}

#[component]
fn SGSectionTitle(title: &'static str, delay_ms: usize) -> impl IntoView {
    view! { <div class="sg-section-title" style=format!("animation-delay:{}ms;", delay_ms)>{title}</div> }
}

#[component]
fn SGCard(
    title: &'static str,
    subtitle: String,
    delay_ms: usize,
    children: Children,
) -> impl IntoView {
    view! {
        <section class="sg-card" style=format!("animation-delay:{}ms;", delay_ms)>
            <h2 class="sg-card-title">{title}</h2>
            <p class="sg-card-sub">{subtitle}</p>
            {children()}
        </section>
    }
}

#[derive(Clone, Copy)]
enum CountUnit {
    Count,
    Percent,
}

#[derive(Clone, Copy)]
enum Palette {
    Neutral,
    Sex,
    Group,
}

fn sex_color(label: &str) -> &'static str {
    let l = label.to_lowercase();
    if l.contains("femme") {
        "rgba(167,139,250,.95)"
    } else if l.contains("homme") {
        "rgba(34,211,238,.92)"
    } else {
        "rgba(148,163,184,.88)"
    }
}

fn compute_global_stats_page_summary(
    deputes: &[DeputeInfo],
    stats: &[DeputeStats],
    period: Period,
) -> GlobalStatsPageSummary {
    let mut groupes: HashMap<String, usize> = HashMap::new();
    let mut partis: HashMap<String, usize> = HashMap::new();
    let mut professions: HashMap<String, usize> = HashMap::new();
    let mut sexes: HashMap<String, usize> = HashMap::new();

    let mut age_values: Vec<u32> = Vec::new();
    let mut age_bins: HashMap<String, usize> = HashMap::new();

    let mut professions_non_renseignees = 0usize;
    let mut sexes_non_renseignes = 0usize;
    let mut age_unknown = 0usize;

    let today = browser_today();

    for d in deputes {
        let group_label = deputy_group_label_info(d);
        *groupes.entry(group_label).or_insert(0) += 1;

        let parti_label = d
            .parti_nom
            .as_deref()
            .map(normalize_label)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Non renseigné".to_string());
        *partis.entry(parti_label).or_insert(0) += 1;

        match d
            .profession
            .as_deref()
            .map(normalize_profession_label)
            .filter(|s| !s.is_empty())
        {
            Some(label) => *professions.entry(label).or_insert(0) += 1,
            None => {
                professions_non_renseignees += 1;
                *professions.entry("Non renseigné".to_string()).or_insert(0) += 1;
            }
        }

        match d
            .sexe
            .as_deref()
            .map(normalize_sexe_label)
            .filter(|s| !s.is_empty())
        {
            Some(label) => *sexes.entry(label).or_insert(0) += 1,
            None => {
                sexes_non_renseignes += 1;
                *sexes.entry("Non renseigné".to_string()).or_insert(0) += 1;
            }
        }

        match d.date_naissance.and_then(|dob| age_years(dob, today)) {
            Some(age) => {
                age_values.push(age);
                *age_bins.entry(age_bin_label(age)).or_insert(0) += 1;
            }
            None => age_unknown += 1,
        }
    }

    let femmes_count = sexes
        .iter()
        .find_map(|(k, v)| if k == "Femme" { Some(*v as u64) } else { None })
        .unwrap_or(0);
    let sexes_known_total: u64 = sexes
        .iter()
        .filter(|(k, _)| k.as_str() != "Non renseigné")
        .map(|(_, v)| *v as u64)
        .sum();

    let mut participation_values_pct: Vec<f64> = Vec::new();
    let mut histogram_counts = [0usize; 10];
    let mut group_acc: HashMap<String, GroupStatsAccumulator> = HashMap::new();
    let mut amendements_deposes_total: u64 = 0;
    let mut amendements_adoptes_total: u64 = 0;
    let mut scrutins_reference_max: u32 = 0;

    for s in stats {
        let part_pct = (s.participation_rate * 100.0).clamp(0.0, 100.0);
        participation_values_pct.push(part_pct);

        let mut idx = (part_pct / 10.0).floor() as usize;
        if idx >= 10 {
            idx = 9;
        }
        histogram_counts[idx] += 1;

        amendements_deposes_total += s.amd_authored as u64;
        amendements_adoptes_total += s.amd_adopted as u64;
        scrutins_reference_max = scrutins_reference_max.max(s.scrutins_eligibles);

        let group_label = deputy_group_label_stats(s);
        let entry = group_acc.entry(group_label).or_default();
        entry.seats += 1;
        entry.participations.push(part_pct);
        entry.scrutins_ref = entry.scrutins_ref.max(s.scrutins_eligibles);
        entry.amd_authored += s.amd_authored as u64;
        entry.amd_adopted += s.amd_adopted as u64;
    }

    let mut histogram = Vec::with_capacity(10);
    for i in 0..10 {
        let from = (i * 10) as f64;
        let to = if i == 9 { 100.0 } else { ((i + 1) * 10) as f64 };
        let label = if i == 9 {
            "90–100%".to_string()
        } else {
            format!("{}–{}", i * 10, (i + 1) * 10)
        };
        histogram.push(HistogramBin {
            label,
            count: histogram_counts[i],
            from,
            to,
        });
    }

    let mut group_activity: Vec<GroupActivityRow> = group_acc
        .into_iter()
        .map(|(group, acc)| {
            let mut vals = acc.participations.clone();
            let median = median_f64(&mut vals);
            let avg = if acc.participations.is_empty() {
                None
            } else {
                Some(acc.participations.iter().sum::<f64>() / acc.participations.len() as f64)
            };
            GroupActivityRow {
                group,
                seats: acc.seats,
                median_participation_pct: median,
                avg_participation_pct: avg,
                scrutins_ref: acc.scrutins_ref,
                amd_authored: acc.amd_authored,
                amd_adopted: acc.amd_adopted,
            }
        })
        .collect();

    group_activity.sort_by(|a, b| {
        b.seats
            .cmp(&a.seats)
            .then_with(|| {
                b.median_participation_pct
                    .unwrap_or(-1.0)
                    .total_cmp(&a.median_participation_pct.unwrap_or(-1.0))
            })
            .then_with(|| a.group.to_lowercase().cmp(&b.group.to_lowercase()))
    });

    let age_moyen = if age_values.is_empty() {
        None
    } else {
        Some(age_values.iter().copied().sum::<u32>() as f64 / age_values.len() as f64)
    };
    let age_median = median_u32(&mut age_values);

    let participation_moyenne_pct = if participation_values_pct.is_empty() {
        None
    } else {
        Some(participation_values_pct.iter().sum::<f64>() / participation_values_pct.len() as f64)
    };
    let participation_mediane_pct = median_f64(&mut participation_values_pct);

    GlobalStatsPageSummary {
        period,
        total_deputes: deputes.len(),
        total_groupes: groupes.len(),
        total_partis: partis.len(),
        age_moyen,
        age_median,
        age_unknown,
        femmes_pct: if sexes_known_total > 0 {
            Some(femmes_count as f64 * 100.0 / sexes_known_total as f64)
        } else {
            None
        },
        participation_mediane_pct,
        participation_moyenne_pct,
        scrutins_reference: scrutins_reference_max,
        amendements_deposes_total,
        amendements_adoptes_total,
        adoption_globale_pct: pct_opt(amendements_adoptes_total, amendements_deposes_total),
        professions_distinctes: professions
            .keys()
            .filter(|k| k.as_str() != "Non renseigné")
            .count(),
        professions_non_renseignees,
        sexes_non_renseignes,
        groupes: sorted_counts(groupes),
        partis: sorted_counts(partis),
        professions: sorted_counts(professions),
        sexes: sorted_counts(sexes),
        ages: sorted_age_bins(age_bins),
        participation_histogram: histogram,
        group_activity,
    }
}

fn deputy_group_label_info(d: &DeputeInfo) -> String {
    d.groupe_abrev
        .as_deref()
        .or(d.groupe_nom.as_deref())
        .map(normalize_label)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Non renseigné".to_string())
}

fn deputy_group_label_stats(d: &DeputeStats) -> String {
    d.groupe_abrev
        .as_deref()
        .or(d.groupe_nom.as_deref())
        .map(normalize_label)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Non renseigné".to_string())
}

fn normalize_label(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn normalize_profession_label(s: &str) -> String {
    let cleaned = normalize_label(s);
    let mut t = cleaned.as_str();

    if let Some(rest) = t.strip_prefix('(') {
        if let Some(idx) = rest.find(')') {
            let after = &rest[idx + 1..];
            let after = after.trim_start();
            let after = after.strip_prefix('-').unwrap_or(after).trim_start();
            if !after.is_empty() {
                t = after;
            }
        }
    }

    t.trim().to_string()
}

fn normalize_sexe_label(s: &str) -> String {
    let v = normalize_label(s);
    let lower = v.to_lowercase();
    if lower.contains("femme") || lower == "f" {
        "Femme".to_string()
    } else if lower.contains("homme") || lower == "h" {
        "Homme".to_string()
    } else {
        v
    }
}

fn sorted_counts(map: HashMap<String, usize>) -> Vec<CountRow> {
    let mut rows: Vec<CountRow> = map
        .into_iter()
        .map(|(label, count)| CountRow { label, count })
        .collect();

    rows.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.label.to_lowercase().cmp(&b.label.to_lowercase()))
    });
    rows
}

fn sorted_age_bins(map: HashMap<String, usize>) -> Vec<CountRow> {
    let order = [
        "18–34 ans",
        "35–44 ans",
        "45–54 ans",
        "55–64 ans",
        "65 ans et +",
    ];

    let mut rows: Vec<CountRow> = order
        .iter()
        .filter_map(|label| {
            map.get(*label)
                .copied()
                .map(|count| CountRow { label: (*label).to_string(), count })
        })
        .collect();

    let mut extras: Vec<CountRow> = map
        .into_iter()
        .filter(|(k, _)| !order.contains(&k.as_str()))
        .map(|(label, count)| CountRow { label, count })
        .collect();

    extras.sort_by(|a, b| a.label.cmp(&b.label));
    rows.extend(extras);
    rows
}

fn top_n_with_other(rows: Vec<CountRow>, max_rows: usize) -> Vec<CountRow> {
    if rows.len() <= max_rows || max_rows == 0 {
        return rows;
    }

    let keep = max_rows.saturating_sub(1).max(1);
    let mut head: Vec<CountRow> = rows.iter().take(keep).cloned().collect();
    let other_count: usize = rows.iter().skip(keep).map(|r| r.count).sum();
    if other_count > 0 {
        head.push(CountRow {
            label: "Autres".to_string(),
            count: other_count,
        });
    }
    head
}

fn browser_today() -> (i32, u32, u32) {
    let d = js_sys::Date::new_0();
    (d.get_full_year() as i32, d.get_month() + 1, d.get_date())
}

fn age_years(dob: NaiveDate, today: (i32, u32, u32)) -> Option<u32> {
    let (y, m, d) = today;
    let today_date = NaiveDate::from_ymd_opt(y, m, d)?;
    if dob > today_date {
        return None;
    }

    let mut age = today_date.year() - dob.year();
    let has_had_birthday = (today_date.month(), today_date.day()) >= (dob.month(), dob.day());
    if !has_had_birthday {
        age -= 1;
    }
    if age < 0 { None } else { Some(age as u32) }
}

fn age_bin_label(age: u32) -> String {
    match age {
        0..=34 => "18–34 ans".to_string(),
        35..=44 => "35–44 ans".to_string(),
        45..=54 => "45–54 ans".to_string(),
        55..=64 => "55–64 ans".to_string(),
        _ => "65 ans et +".to_string(),
    }
}

fn median_u32(values: &mut [u32]) -> Option<u32> {
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    let mid = values.len() / 2;
    if values.len() % 2 == 1 {
        Some(values[mid])
    } else {
        Some((values[mid - 1] + values[mid]) / 2)
    }
}

fn median_f64(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.total_cmp(b));
    let mid = values.len() / 2;
    if values.len() % 2 == 1 {
        Some(values[mid])
    } else {
        Some((values[mid - 1] + values[mid]) / 2.0)
    }
}

fn pct_opt(num: impl Into<u64>, den: impl Into<u64>) -> Option<f64> {
    let num = num.into();
    let den = den.into();
    if den == 0 {
        None
    } else {
        Some(num as f64 * 100.0 / den as f64)
    }
}

fn fmt_int(v: u64) -> String {
    let s = v.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(' ');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn fmt_pct1(v: f64) -> String {
    format!("{v:.1}%")
}

fn fmt_pct0(v: f64) -> String {
    format!("{v:.0}%")
}

fn fmt_age1(v: f64) -> String {
    format!("{v:.1} ans")
}

fn palette_color(palette: Palette, i: usize, label: &str) -> &'static str {
    match palette {
        Palette::Neutral => {
            const COLORS: [&str; 8] = [
                "rgba(99,102,241,.85)",
                "rgba(14,165,233,.85)",
                "rgba(16,185,129,.85)",
                "rgba(245,158,11,.85)",
                "rgba(236,72,153,.85)",
                "rgba(168,85,247,.85)",
                "rgba(244,63,94,.85)",
                "rgba(34,197,94,.85)",
            ];
            COLORS[i % COLORS.len()]
        }
        Palette::Sex => {
            let l = label.to_lowercase();
            if l.contains("femme") {
                "rgba(236,72,153,.88)"
            } else if l.contains("homme") {
                "rgba(99,102,241,.88)"
            } else if l.contains("non renseign") {
                "rgba(148,163,184,.8)"
            } else {
                "rgba(16,185,129,.85)"
            }
        }
        Palette::Group => group_palette_color(label),
    }
}

fn group_palette_color(label: &str) -> &'static str {
    match label.trim() {
        "RN" => "rgba(59,130,246,.92)",
        "EPR" => "rgba(249,115,22,.92)",
        "LFI-NFP" | "LFI" => "rgba(239,68,68,.92)",
        "SOC" => "rgba(236,72,153,.92)",
        "EcoS" | "ECO" => "rgba(34,197,94,.92)",
        "DR" | "LR" | "UDR" => "rgba(37,99,235,.92)",
        "DEM" | "MoDem" => "rgba(14,165,233,.92)",
        "HOR" => "rgba(168,85,247,.92)",
        "LIOT" => "rgba(16,185,129,.92)",
        "GDR" => "rgba(220,38,38,.92)",
        "NI" | "Non inscrit" | "Non-inscrits" | "Non renseigné" => "rgba(148,163,184,.85)",
        _ => {
            const COLORS: [&str; 8] = [
                "rgba(99,102,241,.85)",
                "rgba(14,165,233,.85)",
                "rgba(16,185,129,.85)",
                "rgba(245,158,11,.85)",
                "rgba(236,72,153,.85)",
                "rgba(168,85,247,.85)",
                "rgba(244,63,94,.85)",
                "rgba(34,197,94,.85)",
            ];
            let idx = label.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize)) % COLORS.len();
            COLORS[idx]
        }
    }
}
