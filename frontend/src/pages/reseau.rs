use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;
use leptos::*;
use leptos_router::A;

use crate::components::period_selector::PeriodSelector;
use crate::models::{CosignNetworkStats, DeputeStats, Period};
use crate::store::use_store;
use crate::utils::app_href;

#[derive(Debug, Clone, Default, PartialEq)]
struct GroupNode {
    key: String,
    label: String,
    full_name: String,
    deputy_count: usize,
    deputies_with_network: usize,
    total_cosignatures: u64,
    in_group_count: u64,
    out_group_count: u64,
}

impl GroupNode {
    fn transpartisan_rate(&self) -> f64 {
        if self.total_cosignatures == 0 {
            0.0
        } else {
            self.out_group_count as f64 / self.total_cosignatures as f64
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct GroupEdge {
    a_key: String,
    a_label: String,
    b_key: String,
    b_label: String,
    a_to_b: u64,
    b_to_a: u64,
    total: u64,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct GroupNetworkSummary {
    groups: Vec<GroupNode>,
    matrix: Vec<Vec<u64>>, // matrice orientée [source][target]
    matrix_max: u64,
    total_deputes: usize,
    deputies_with_network: usize,
    deputies_without_network: usize,
    total_cosignatures: u64,
    total_in_group: u64,
    total_out_group: u64,
    top_edges: Vec<GroupEdge>,
    period_start: Option<NaiveDate>,
    period_end: Option<NaiveDate>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct PartnerFlow {
    key: String,
    label: String,
    full_name: String,
    count: u64,
    pct: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct BridgeDeputy {
    deputy_id: String,
    nom_complet: String,
    out_group_count: u64,
    total_cosignatures: u64,
    transpartisan_rate: f64,
    unique_cosignataires: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct FocusGroupData {
    group: GroupNode,
    incoming_total: u64,
    outgoing_total: u64,
    self_total: u64,
    incoming_partners: Vec<PartnerFlow>,
    outgoing_partners: Vec<PartnerFlow>,
    symmetric_partners: Vec<PartnerFlow>,
    bridge_deputies: Vec<BridgeDeputy>,
}

#[derive(Debug, Clone, Default)]
struct TmpGroup {
    label: String,
    full_name: String,
    deputy_count: usize,
    deputies_with_network: usize,
    total_cosignatures: u64,
    in_group_count: u64,
    out_group_count: u64,
}

#[component]
pub fn ReseauPage() -> impl IntoView {
    let store = use_store();
    let (period, set_period) = create_signal(Period::P180);
    let (selected_group, set_selected_group) = create_signal::<Option<String>>(None);

    let store_for_stats = store.clone();
    let stats_result = create_memo(move |_| store_for_stats.stats_for(period.get()).get());

    let stats_data =
        create_memo(move |_| stats_result.get().and_then(|r| r.ok()).unwrap_or_default());

    let summary = create_memo(move |_| build_group_network(&stats_data.get()));

    let summary_for_default = summary.clone();
    create_effect(move |_| {
        let s = summary_for_default.get();
        let current = selected_group.get();

        if s.groups.is_empty() {
            if current.is_some() {
                set_selected_group.set(None);
            }
            return;
        }

        let still_exists = current
            .as_ref()
            .map(|g| s.groups.iter().any(|x| x.key == *g))
            .unwrap_or(false);

        if !still_exists {
            set_selected_group.set(s.groups.first().map(|g| g.key.clone()));
        }
    });

    let focus_data = create_memo(move |_| {
        let key = selected_group.get();
        let s = summary.get();
        let data = stats_data.get();
        key.and_then(|k| build_focus_group(&s, &data, &k))
    });

    view! {
        <div class="reveal" style="position:relative;">
            <div
                style="position:absolute;inset:0;pointer-events:none;opacity:.22;z-index:0;background-image:linear-gradient(var(--bg-border) 1px, transparent 1px),linear-gradient(90deg, var(--bg-border) 1px, transparent 1px);background-size:40px 40px;border-radius:14px;"
            ></div>

            <div style="position:relative;z-index:1;display:flex;flex-direction:column;gap:1rem;">
                <section style="padding:1rem 1rem 0.9rem 1rem;background:linear-gradient(180deg, rgba(34,211,238,0.05), rgba(34,211,238,0.01));border:1px solid var(--bg-border);border-radius:12px;">
                    <div style="display:flex;justify-content:space-between;align-items:flex-start;gap:1rem;flex-wrap:wrap;">
                        <div style="max-width:920px;">
                            <h1 style="margin:0 0 .35rem 0;font-size:1.18rem;font-weight:700;letter-spacing:.01em;">
                                "Réseau de co-signatures entre groupes"
                            </h1>
                            <p style="margin:0;color:var(--text-muted);font-size:.8rem;line-height:1.45;">
                                "Vue agrégée des co-signatures d’amendements à partir des réseaux individuels calculés dans le pipeline. "
                                "La matrice est orientée (groupe source → groupe cible) et permet d’identifier les liens intra-groupe, les ponts transpartisans et les groupes les plus connectés."
                            </p>
                            {move || store.status.get().and_then(|r| r.ok()).map(|st| view! {
                                <p style="margin:.45rem 0 0 0;font-size:.74rem;color:var(--text-muted);">
                                    "Dernière mise à jour : "
                                    <strong style="color:var(--text-secondary);">{st.last_update_readable}</strong>
                                </p>
                            })}
                        </div>
                        <div style="display:flex;align-items:center;gap:.6rem;flex-wrap:wrap;">
                            <PeriodSelector period=period set_period=set_period />
                            <A href=app_href("/methodologie") class="btn" attr:style="text-decoration:none;">"Méthode"</A>
                        </div>
                    </div>
                </section>

                {move || match stats_result.get() {
                    None => view! {
                        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:1rem;">
                            {(0..6).map(|_| view! {
                                <div style="height:92px;border-radius:10px;background:var(--bg-secondary);border:1px solid var(--bg-border);"></div>
                            }).collect_view()}
                        </div>
                    }.into_view(),
                    Some(Err(e)) => view! {
                        <div style="padding:1rem;border:1px solid var(--danger);border-radius:10px;background:rgba(239,68,68,.08);color:var(--danger);">
                            {format!("Erreur de chargement des statistiques ({}) : {}", period.get().label(), e)}
                        </div>
                    }.into_view(),
                    Some(Ok(_)) => {
                        let s = summary.get();
                        let focus = focus_data.get();
                        let selected_key = selected_group.get();

                        if s.groups.is_empty() {
                            view! {
                                <div style="padding:1rem;border:1px solid var(--bg-border);border-radius:10px;background:var(--bg-secondary);color:var(--text-muted);">
                                    "Aucune donnée de réseau exploitable sur cette période."
                                </div>
                            }.into_view()
                        } else {

                        let period_label = match (s.period_start, s.period_end) {
                            (Some(a), Some(b)) => format!("{} → {}", a, b),
                            _ => period.get().label().to_string(),
                        };

                        let trans_share = if s.total_cosignatures > 0 {
                            s.total_out_group as f64 / s.total_cosignatures as f64
                        } else { 0.0 };

                        view! {
                            <>
                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(185px,1fr));gap:.85rem;">
                                    <MetricCard label="Période effective" value=period_label sub="déduite du dataset chargé".to_string() />
                                    <MetricCard label="Groupes observés" value=s.groups.len().to_string() sub="avec non-inscrits si présents".to_string() />
                                    <MetricCard label="Députés couverts" value=format!("{} / {}", s.deputies_with_network, s.total_deputes) sub=format!("{} sans réseau détaillé", s.deputies_without_network) />
                                    <MetricCard label="Volume total (dir.)" value=fmt_u64(s.total_cosignatures) sub="somme des réseaux individuels".to_string() />
                                    <MetricCard label="Part intra-groupe" value=fmt_pct1_ratio(if s.total_cosignatures > 0 { s.total_in_group as f64 / s.total_cosignatures as f64 } else { 0.0 }) sub=fmt_u64(s.total_in_group) />
                                    <MetricCard label="Part transpartisane" value=fmt_pct1_ratio(trans_share) sub=fmt_u64(s.total_out_group) />
                                </div>

                                <div class="reseau-main-grid" style="display:grid;grid-template-columns:minmax(260px,320px) 1fr;gap:1rem;align-items:start;">
                                    <GroupListPanel
                                        summary=s.clone()
                                        selected_key=selected_key.clone()
                                        set_selected=set_selected_group
                                    />

                                    <div style="display:flex;flex-direction:column;gap:1rem;min-width:0;">
                                        <MatrixSection
                                            summary=s.clone()
                                            selected_key=selected_key.clone()
                                            set_selected=set_selected_group
                                        />
                                        <TopEdgesSection summary=s.clone() />
                                        <FocusGroupPanel focus=focus selected_key=selected_key />
                                    </div>
                                </div>

                                <div style="padding:.8rem .95rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:10px;font-size:.76rem;color:var(--text-muted);line-height:1.45;">
                                    <strong style="color:var(--text-secondary);">"Lecture"</strong>
                                    " : les volumes affichés sont issus des réseaux de co-signatures individuels (orientés source → cible). "
                                    "Ils mesurent une intensité relationnelle agrégée, pas un graphe social “unique” dédupliqué amendement par amendement."
                                </div>
                            </>
                        }.into_view()
                        }
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn MetricCard(label: &'static str, value: String, sub: String) -> impl IntoView {
    view! {
        <div style="padding:.85rem .95rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:10px;">
            <div style="font-size:.68rem;text-transform:uppercase;letter-spacing:.08em;color:var(--text-muted);margin-bottom:.35rem;font-weight:600;">{label}</div>
            <div style="font-size:1.02rem;font-weight:700;color:var(--text-primary);margin-bottom:.18rem;line-height:1.15;font-variant-numeric:tabular-nums;">{value}</div>
            <div style="font-size:.74rem;color:var(--text-muted);line-height:1.35;">{sub}</div>
        </div>
    }
}

#[component]
fn GroupListPanel(
    summary: GroupNetworkSummary,
    selected_key: Option<String>,
    set_selected: WriteSignal<Option<String>>,
) -> impl IntoView {
    let max_total = summary
        .groups
        .iter()
        .map(|g| g.total_cosignatures)
        .max()
        .unwrap_or(1);

    view! {
        <section style="padding:1rem;background:rgba(0,0,0,.12);border:1px solid var(--bg-border);border-radius:12px;display:flex;flex-direction:column;gap:.8rem;position:sticky;top:70px;">
            <div>
                <h2 style="margin:0 0 .25rem 0;font-size:.82rem;text-transform:uppercase;letter-spacing:.08em;color:var(--text-muted);font-weight:700;">"Groupes"</h2>
                <p style="margin:0;color:var(--text-muted);font-size:.75rem;line-height:1.35;">
                    "Sélectionnez un groupe pour mettre en évidence ses flux et ses principaux relais transpartisans."
                </p>
            </div>

            <div style="display:flex;flex-direction:column;gap:.45rem;max-height:70vh;overflow:auto;padding-right:.1rem;">
                {summary.groups.into_iter().map(|g| {
                    let is_selected = selected_key.as_deref() == Some(g.key.as_str());
                    let dot = group_color(&g.key);
                    let bar_pct = if max_total > 0 {
                        (g.total_cosignatures as f64 * 100.0 / max_total as f64).clamp(0.0, 100.0)
                    } else { 0.0 };
                    let trans = g.transpartisan_rate();
                    let key = g.key.clone();
                    let full_name = g.full_name.clone();
                    let label = g.label.clone();
                    let total_label = fmt_compact_u64(g.total_cosignatures);
                    let row_style = if is_selected {
                        "display:flex;flex-direction:column;gap:.35rem;padding:.55rem .6rem;border-radius:8px;border:1px solid rgba(34,211,238,.35);background:rgba(34,211,238,.06);cursor:pointer;"
                    } else {
                        "display:flex;flex-direction:column;gap:.35rem;padding:.55rem .6rem;border-radius:8px;border:1px solid transparent;background:transparent;cursor:pointer;"
                    };

                    view! {
                        <button
                            on:click=move |_| set_selected.set(Some(key.clone()))
                            title=full_name.clone()
                            style=row_style
                        >
                            <div style="display:flex;align-items:center;gap:.5rem;">
                                <span style=format!("display:inline-block;width:10px;height:10px;border-radius:999px;background:{};box-shadow:0 0 10px {}55;flex:0 0 auto;", dot, dot)></span>
                                <span style="font-size:.78rem;font-weight:600;color:var(--text-primary);flex:1;text-align:left;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{label.clone()}</span>
                                <span style="font-size:.7rem;color:var(--text-muted);font-variant-numeric:tabular-nums;">{format!("{} d.", g.deputy_count)}</span>
                            </div>
                            <div style="height:4px;background:rgba(255,255,255,.04);border-radius:99px;overflow:hidden;">
                                <div style=format!("height:100%;width:{:.3}%;background:{};opacity:.85;", bar_pct, dot)></div>
                            </div>
                            <div style="display:flex;justify-content:space-between;gap:.5rem;font-size:.69rem;color:var(--text-muted);font-variant-numeric:tabular-nums;">
                                <span>{format!("Volume {}", total_label)}</span>
                                <span>{format!("Trans {}", fmt_pct1_ratio(trans))}</span>
                            </div>
                        </button>
                    }
                }).collect_view()}
            </div>
        </section>
    }
}

#[component]
fn MatrixSection(
    summary: GroupNetworkSummary,
    selected_key: Option<String>,
    set_selected: WriteSignal<Option<String>>,
) -> impl IntoView {
    let selected_idx = selected_key
        .as_ref()
        .and_then(|k| summary.groups.iter().position(|g| &g.key == k));

    let cell_max = summary.matrix_max.max(1);

    view! {
        <section style="background:rgba(0,0,0,.12);border:1px solid var(--bg-border);border-radius:12px;overflow:hidden;">
            <div style="padding:.85rem 1rem;border-bottom:1px solid var(--bg-border);display:flex;justify-content:space-between;align-items:flex-start;gap:1rem;flex-wrap:wrap;">
                <div>
                    <h2 style="margin:0 0 .25rem 0;font-size:.9rem;font-weight:700;color:var(--text-primary);">"Matrice des flux de co-signatures (orientée)"</h2>
                    <p style="margin:0;color:var(--text-muted);font-size:.75rem;line-height:1.35;">
                        "Lignes = groupe source, colonnes = groupe cible. Diagonale = co-signatures intra-groupe."
                    </p>
                </div>
                <div style="display:flex;gap:.4rem;flex-wrap:wrap;align-items:center;">
                    <span style="font-size:.68rem;color:var(--text-muted);text-transform:uppercase;letter-spacing:.08em;">"Sélection"</span>
                    {selected_key.map(|k| view! {
                        <span style="display:inline-flex;align-items:center;gap:.35rem;padding:.18rem .55rem;border-radius:999px;border:1px solid var(--bg-border);background:var(--bg-secondary);font-size:.72rem;color:var(--text-secondary);">
                            <span style=format!("width:7px;height:7px;border-radius:999px;background:{};display:inline-block;", group_color(&k))></span>
                            {k}
                        </span>
                    })}
                </div>
            </div>

            <div style="overflow:auto;max-width:100%;">
                <table style="width:100%;border-collapse:separate;border-spacing:0;min-width:760px;font-size:.74rem;">
                    <thead>
                        <tr>
                            <th style="position:sticky;left:0;z-index:3;background:var(--bg-secondary);padding:.55rem .6rem;text-align:left;border-bottom:1px solid var(--bg-border);border-right:1px solid var(--bg-border);font-size:.68rem;color:var(--text-muted);text-transform:uppercase;letter-spacing:.08em;">"Source ↓ / Cible →"</th>
                            {summary.groups.iter().enumerate().map(|(j, g)| {
                                let is_selected_col = selected_idx == Some(j);
                                let key = g.key.clone();
                                let style = if is_selected_col {
                                    format!("padding:.45rem .4rem;text-align:center;border-bottom:1px solid var(--bg-border);background:rgba(34,211,238,.07);min-width:58px;")
                                } else {
                                    "padding:.45rem .4rem;text-align:center;border-bottom:1px solid var(--bg-border);background:rgba(255,255,255,.01);min-width:58px;".to_string()
                                };
                                view! {
                                    <th style=style>
                                        <button
                                            on:click=move |_| set_selected.set(Some(key.clone()))
                                            title=g.full_name.clone()
                                            style="background:none;border:none;cursor:pointer;color:var(--text-secondary);font-size:.68rem;font-weight:700;letter-spacing:.03em;padding:0;"
                                        >
                                            <span style=format!("display:inline-flex;align-items:center;gap:.28rem;")>
                                                <span style=format!("width:7px;height:7px;border-radius:999px;background:{};display:inline-block;", group_color(&g.key))></span>
                                                {g.label.clone()}
                                            </span>
                                        </button>
                                    </th>
                                }
                            }).collect_view()}
                        </tr>
                    </thead>
                    <tbody>
                        {summary.groups.iter().enumerate().map(|(i, row_g)| {
                            let row_selected = selected_idx == Some(i);
                            let row_key = row_g.key.clone();
                            view! {
                                <tr>
                                    <th
                                        style=if row_selected {
                                            "position:sticky;left:0;z-index:2;background:rgba(34,211,238,.07);padding:.45rem .6rem;text-align:left;border-right:1px solid var(--bg-border);border-bottom:1px solid var(--bg-border);".to_string()
                                        } else {
                                            "position:sticky;left:0;z-index:2;background:var(--bg-secondary);padding:.45rem .6rem;text-align:left;border-right:1px solid var(--bg-border);border-bottom:1px solid var(--bg-border);".to_string()
                                        }
                                    >
                                        <button
                                            on:click=move |_| set_selected.set(Some(row_key.clone()))
                                            title=row_g.full_name.clone()
                                            style="display:flex;align-items:center;gap:.4rem;background:none;border:none;padding:0;cursor:pointer;color:var(--text-secondary);font-weight:600;font-size:.72rem;"
                                        >
                                            <span style=format!("width:8px;height:8px;border-radius:999px;background:{};display:inline-block;", group_color(&row_g.key))></span>
                                            <span>{row_g.label.clone()}</span>
                                        </button>
                                    </th>

                                    {summary.matrix[i].iter().enumerate().map(|(j, v)| {
                                        let is_diag = i == j;
                                        let selected_line = selected_idx.map(|idx| idx == i || idx == j).unwrap_or(false);
                                        let intensity = (*v as f64 / cell_max as f64).sqrt();
                                        let alpha = if *v == 0 { 0.0 } else { 0.08 + 0.62 * intensity };
                                        let base = if is_diag {
                                            format!("rgba(34,211,238,{:.3})", (alpha + 0.10).min(0.86))
                                        } else {
                                            format!("rgba(148,163,184,{:.3})", alpha.min(0.72))
                                        };
                                        let border_color = if selected_line {
                                            "rgba(34,211,238,.18)"
                                        } else {
                                            "rgba(255,255,255,.03)"
                                        };
                                        let display = if *v == 0 { "·".to_string() } else { fmt_compact_u64(*v) };
                                        let title = format!("{} → {} : {}", row_g.label, summary.groups[j].label, fmt_u64(*v));
                                        view! {
                                            <td
                                                title=title
                                                style=format!(
                                                    "padding:.28rem .3rem;text-align:center;border-bottom:1px solid {};border-right:1px solid {};background:{};font-variant-numeric:tabular-nums;{}",
                                                    border_color,
                                                    border_color,
                                                    base,
                                                    if is_diag { "font-weight:700;color:var(--text-primary);" } else { "color:var(--text-secondary);" }
                                                )
                                            >
                                                <span style="font-size:.68rem;">{display}</span>
                                            </td>
                                        }
                                    }).collect_view()}
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>

            <div style="padding:.65rem 1rem;border-top:1px solid var(--bg-border);display:flex;flex-wrap:wrap;gap:.9rem;font-size:.7rem;color:var(--text-muted);align-items:center;">
                <span style="display:inline-flex;align-items:center;gap:.35rem;">
                    <span style="width:12px;height:12px;border:1px solid rgba(255,255,255,.06);background:rgba(34,211,238,.25);display:inline-block;border-radius:3px;"></span>
                    "diagonale = intra-groupe"
                </span>
                <span style="display:inline-flex;align-items:center;gap:.35rem;">
                    <span style="width:12px;height:12px;border:1px solid rgba(255,255,255,.06);background:rgba(148,163,184,.35);display:inline-block;border-radius:3px;"></span>
                    "hors groupe (intensité ∝ volume)"
                </span>
                <span>"Cliquez sur un groupe (ligne/colonne) pour le focus."</span>
            </div>
        </section>
    }
}

#[component]
fn TopEdgesSection(summary: GroupNetworkSummary) -> impl IntoView {
    let rows = summary.top_edges.into_iter().take(14).collect::<Vec<_>>();
    let max_val = rows.iter().map(|e| e.total).max().unwrap_or(1);

    view! {
        <section style="background:rgba(0,0,0,.12);border:1px solid var(--bg-border);border-radius:12px;overflow:hidden;">
            <div style="padding:.85rem 1rem;border-bottom:1px solid var(--bg-border);">
                <h2 style="margin:0 0 .25rem 0;font-size:.88rem;font-weight:700;">"Liens inter-groupes les plus forts"</h2>
                <p style="margin:0;color:var(--text-muted);font-size:.75rem;line-height:1.35;">
                    "Classement symétrisé (A→B + B→A) pour repérer les paires de groupes les plus connectées."
                </p>
            </div>

            {if rows.is_empty() {
                view! { <div style="padding:1rem;color:var(--text-muted);font-size:.78rem;">"Aucun lien inter-groupe disponible sur cette période."</div> }.into_view()
            } else {
                view! {
                    <div style="padding:.75rem 1rem 1rem 1rem;display:flex;flex-direction:column;gap:.45rem;">
                        {rows.into_iter().map(|e| {
                            let pct = (e.total as f64 * 100.0 / max_val as f64).clamp(0.0, 100.0);
                            let left_color = group_color(&e.a_key);
                            let right_color = group_color(&e.b_key);
                            view! {
                                <div class="reseau-edge-row" style="display:grid;grid-template-columns:minmax(180px,1.2fr) minmax(120px,2fr) auto;gap:.7rem;align-items:center;">
                                    <div style="font-size:.77rem;color:var(--text-secondary);display:flex;align-items:center;gap:.45rem;min-width:0;">
                                        <span style=format!("width:8px;height:8px;border-radius:999px;background:{};display:inline-block;flex:0 0 auto;", left_color)></span>
                                        <span style="white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{e.a_label.clone()}</span>
                                        <span style="color:var(--text-muted);">"↔"</span>
                                        <span style=format!("width:8px;height:8px;border-radius:999px;background:{};display:inline-block;flex:0 0 auto;", right_color)></span>
                                        <span style="white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{e.b_label.clone()}</span>
                                    </div>
                                    <div style="height:8px;background:rgba(255,255,255,.03);border-radius:999px;overflow:hidden;border:1px solid rgba(255,255,255,.04);position:relative;">
                                        <div style=format!("height:100%;width:{:.3}%;background:linear-gradient(90deg, {} 0%, {} 100%);opacity:.9;", pct, left_color, right_color)></div>
                                    </div>
                                    <div style="text-align:right;font-size:.72rem;color:var(--text-muted);font-variant-numeric:tabular-nums;white-space:nowrap;">
                                        <strong style="color:var(--text-primary);">{fmt_u64(e.total)}</strong>
                                        <span style="margin-left:.4rem;">{format!("{} / {}", fmt_compact_u64(e.a_to_b), fmt_compact_u64(e.b_to_a))}</span>
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }.into_view()
            }}
        </section>
    }
}

#[component]
fn FocusGroupPanel(focus: Option<FocusGroupData>, selected_key: Option<String>) -> impl IntoView {
    match focus {
        None => {
            let msg = if selected_key.is_some() {
                "Aucune donnée exploitable pour ce groupe sur la période sélectionnée."
            } else {
                "Sélectionnez un groupe pour afficher son profil détaillé."
            };
            view! {
                <section style="background:rgba(0,0,0,.12);border:1px solid var(--bg-border);border-radius:12px;padding:1rem;">
                    <h2 style="margin:0 0 .35rem 0;font-size:.88rem;font-weight:700;">"Focus groupe"</h2>
                    <p style="margin:0;color:var(--text-muted);font-size:.78rem;line-height:1.4;">{msg}</p>
                </section>
            }.into_view()
        }
        Some(f) => {
            let dot = group_color(&f.group.key);
            let trans = f.group.transpartisan_rate();
            let bridge_max = f
                .bridge_deputies
                .iter()
                .map(|d| d.out_group_count)
                .max()
                .unwrap_or(1);

            view! {
                <section style="background:rgba(0,0,0,.12);border:1px solid var(--bg-border);border-radius:12px;overflow:hidden;">
                    <div style="padding:.9rem 1rem;border-bottom:1px solid var(--bg-border);display:flex;justify-content:space-between;align-items:flex-start;gap:1rem;flex-wrap:wrap;">
                        <div>
                            <h2 style="margin:0 0 .25rem 0;font-size:.9rem;font-weight:700;color:var(--text-primary);display:flex;align-items:center;gap:.45rem;">
                                <span style=format!("width:10px;height:10px;border-radius:999px;background:{};display:inline-block;box-shadow:0 0 10px {}55;", dot, dot)></span>
                                {format!("Focus groupe — {}", f.group.label)}
                            </h2>
                            <p style="margin:0;color:var(--text-muted);font-size:.75rem;line-height:1.35;max-width:800px;">
                                {f.group.full_name.clone()}
                            </p>
                        </div>
                        <div style="display:flex;gap:.45rem;flex-wrap:wrap;">
                            <Chip label=format!("{} députés", f.group.deputy_count) />
                            <Chip label=format!("Trans {}", fmt_pct1_ratio(trans)) />
                            <Chip label=format!("Intra {}", fmt_compact_u64(f.self_total)) />
                            <Chip label=format!("Hors groupe {}", fmt_compact_u64(f.outgoing_total)) />
                        </div>
                    </div>

                    <div style="padding:1rem;display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:1rem;align-items:start;">
                        <div style="display:flex;flex-direction:column;gap:.75rem;">
                            <MiniStat label="Volume total" value=fmt_u64(f.group.total_cosignatures) sub="réseaux individuels du groupe".to_string() />
                            <MiniStat label="Flux sortants inter-groupes" value=fmt_u64(f.outgoing_total) sub="source = ce groupe".to_string() />
                            <MiniStat label="Flux entrants inter-groupes" value=fmt_u64(f.incoming_total) sub="cible = ce groupe".to_string() />
                            <MiniStat label="Députés avec réseau" value=format!("{} / {}", f.group.deputies_with_network, f.group.deputy_count) sub="complétude de la période".to_string() />
                        </div>

                        <PartnerListCard
                            title="Partenaires principaux (sortants)"
                            subtitle="Vers quels groupes les députés de ce groupe co-signent le plus"
                            rows=f.outgoing_partners.clone()
                            accent=dot.to_string()
                        />

                        <PartnerListCard
                            title="Partenaires principaux (entrants)"
                            subtitle="Quels groupes co-signent le plus avec ce groupe (vu depuis leurs réseaux)"
                            rows=f.incoming_partners.clone()
                            accent=dot.to_string()
                        />

                        <PartnerListCard
                            title="Partenaires principaux (symétrisé)"
                            subtitle="A↔B = flux sortants + entrants pour une lecture relationnelle"
                            rows=f.symmetric_partners.clone()
                            accent=dot.to_string()
                        />
                    </div>

                    <div style="padding:0 1rem 1rem 1rem;">
                        <div style="border:1px solid var(--bg-border);border-radius:10px;overflow:hidden;background:var(--bg-secondary);">
                            <div style="padding:.75rem .9rem;border-bottom:1px solid var(--bg-border);display:flex;justify-content:space-between;align-items:flex-start;gap:1rem;flex-wrap:wrap;">
                                <div>
                                    <h3 style="margin:0 0 .2rem 0;font-size:.82rem;font-weight:700;">"Députés du groupe les plus transpartisans (hors-groupe)"</h3>
                                    <p style="margin:0;color:var(--text-muted);font-size:.73rem;line-height:1.35;">
                                        "Classement par volume de co-signatures hors groupe dans les réseaux individuels."
                                    </p>
                                </div>
                            </div>

                            {if f.bridge_deputies.is_empty() {
                                view! { <div style="padding:.85rem .9rem;color:var(--text-muted);font-size:.76rem;">"Aucun député classable sur cette période."</div> }.into_view()
                            } else {
                                view! {
                                    <div style="display:flex;flex-direction:column;">
                                        {f.bridge_deputies.into_iter().take(12).enumerate().map(|(idx, d)| {
                                            let pct = (d.out_group_count as f64 * 100.0 / bridge_max as f64).clamp(0.0, 100.0);
                                            view! {
                                                <div style="padding:.65rem .9rem;border-top:1px solid rgba(255,255,255,.03);display:grid;grid-template-columns:auto minmax(190px,1.5fr) minmax(120px,1fr) auto;gap:.7rem;align-items:center;">
                                                    <div style="font-size:.72rem;color:var(--text-muted);font-variant-numeric:tabular-nums;width:1.6rem;">{format!("#{:02}", idx + 1)}</div>
                                                    <div style="min-width:0;display:flex;flex-direction:column;gap:.16rem;">
                                                        <A href=app_href(&format!("/depute/{}", d.deputy_id)) attr:style="color:var(--text-secondary);text-decoration:none;font-weight:600;font-size:.76rem;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">
                                                            {d.nom_complet.clone()}
                                                        </A>
                                                        <div style="font-size:.68rem;color:var(--text-muted);font-variant-numeric:tabular-nums;display:flex;gap:.65rem;flex-wrap:wrap;">
                                                            <span>{format!("total {}", fmt_compact_u64(d.total_cosignatures))}</span>
                                                            <span>{format!("{} cosignataires uniques", d.unique_cosignataires)}</span>
                                                        </div>
                                                    </div>
                                                    <div style="height:7px;background:rgba(255,255,255,.03);border:1px solid rgba(255,255,255,.04);border-radius:999px;overflow:hidden;">
                                                        <div style=format!("height:100%;width:{:.3}%;background:{};opacity:.9;", pct, dot)></div>
                                                    </div>
                                                    <div style="text-align:right;font-size:.72rem;color:var(--text-muted);font-variant-numeric:tabular-nums;white-space:nowrap;">
                                                        <strong style="color:var(--text-primary);">{fmt_compact_u64(d.out_group_count)}</strong>
                                                        <span style="margin-left:.35rem;">{fmt_pct1_ratio(d.transpartisan_rate)}</span>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_view()
                            }}
                        </div>
                    </div>
                </section>
            }.into_view()
        }
    }
}

#[component]
fn Chip(label: String) -> impl IntoView {
    view! {
        <span style="display:inline-flex;align-items:center;padding:.18rem .55rem;border-radius:999px;border:1px solid var(--bg-border);background:var(--bg-secondary);font-size:.72rem;color:var(--text-secondary);font-variant-numeric:tabular-nums;">
            {label}
        </span>
    }
}

#[component]
fn MiniStat(label: &'static str, value: String, sub: String) -> impl IntoView {
    view! {
        <div style="padding:.75rem .85rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:10px;">
            <div style="font-size:.68rem;text-transform:uppercase;letter-spacing:.08em;color:var(--text-muted);margin-bottom:.25rem;">{label}</div>
            <div style="font-size:.95rem;font-weight:700;color:var(--text-primary);margin-bottom:.18rem;font-variant-numeric:tabular-nums;">{value}</div>
            <div style="font-size:.72rem;color:var(--text-muted);line-height:1.35;">{sub}</div>
        </div>
    }
}

#[component]
fn PartnerListCard(
    title: &'static str,
    subtitle: &'static str,
    rows: Vec<PartnerFlow>,
    accent: String,
) -> impl IntoView {
    let rows = rows.into_iter().take(8).collect::<Vec<_>>();
    let max_v = rows.iter().map(|r| r.count).max().unwrap_or(1);
    view! {
        <div style="padding:.85rem .9rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:10px;display:flex;flex-direction:column;gap:.55rem;">
            <div>
                <h3 style="margin:0 0 .15rem 0;font-size:.78rem;font-weight:700;">{title}</h3>
                <p style="margin:0;color:var(--text-muted);font-size:.7rem;line-height:1.35;">{subtitle}</p>
            </div>

            {if rows.is_empty() {
                view! { <p style="margin:0;color:var(--text-muted);font-size:.74rem;">"Aucun flux disponible."</p> }.into_view()
            } else {
                view! {
                    <div style="display:flex;flex-direction:column;gap:.4rem;">
                        {rows.into_iter().map(|r| {
                            let pct = (r.count as f64 * 100.0 / max_v as f64).clamp(0.0, 100.0);
                            view! {
                                <div class="reseau-partner-row" style="display:grid;grid-template-columns:minmax(90px,1.3fr) minmax(80px,2fr) auto;gap:.55rem;align-items:center;">
                                    <div title=r.full_name.clone() style="font-size:.74rem;color:var(--text-secondary);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;display:flex;align-items:center;gap:.35rem;">
                                        <span style=format!("width:7px;height:7px;border-radius:999px;background:{};display:inline-block;", group_color(&r.key))></span>
                                        {r.label.clone()}
                                    </div>
                                    <div style="height:6px;background:rgba(255,255,255,.03);border-radius:999px;overflow:hidden;border:1px solid rgba(255,255,255,.04);">
                                        <div style=format!("height:100%;width:{:.3}%;background:{};opacity:.85;", pct, accent)></div>
                                    </div>
                                    <div style="font-size:.71rem;color:var(--text-muted);font-variant-numeric:tabular-nums;text-align:right;white-space:nowrap;">
                                        <strong style="color:var(--text-primary);">{fmt_compact_u64(r.count)}</strong>
                                        <span style="margin-left:.35rem;">{fmt_pct1(r.pct)}</span>
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }.into_view()
            }}
        </div>
    }
}

fn build_group_network(stats: &[DeputeStats]) -> GroupNetworkSummary {
    let mut groups: BTreeMap<String, TmpGroup> = BTreeMap::new();
    let mut matrix_map: HashMap<(String, String), u64> = HashMap::new();

    let mut total_cosignatures: u64 = 0;
    let mut total_in_group: u64 = 0;
    let mut total_out_group: u64 = 0;
    let mut deputies_with_network = 0usize;
    let mut period_start: Option<NaiveDate> = None;
    let mut period_end: Option<NaiveDate> = None;

    for d in stats {
        let gk = normalize_group_key(d.groupe_abrev.as_deref());
        let glabel = display_group_label(d.groupe_abrev.as_deref(), d.groupe_nom.as_deref());
        let gfull = d.groupe_nom.clone().unwrap_or_else(|| glabel.clone());

        let entry = groups.entry(gk.clone()).or_default();
        if entry.label.is_empty() {
            entry.label = glabel;
        }
        if entry.full_name.is_empty() {
            entry.full_name = gfull;
        }
        entry.deputy_count += 1;

        period_start = Some(
            period_start
                .map(|x| x.min(d.period_start))
                .unwrap_or(d.period_start),
        );
        period_end = Some(
            period_end
                .map(|x| x.max(d.period_end))
                .unwrap_or(d.period_end),
        );

        if let Some(net) = &d.cosign_network {
            deputies_with_network += 1;
            apply_cosign_network(&mut groups, &mut matrix_map, &gk, net);
            total_cosignatures += net.total_cosignatures as u64;
            total_in_group += net.in_group_count as u64;
            total_out_group += net.out_group_count as u64;
        }
    }

    let mut group_vec: Vec<GroupNode> = groups
        .into_iter()
        .map(|(key, g)| GroupNode {
            key: key.clone(),
            label: if g.label.is_empty() {
                key.clone()
            } else {
                g.label
            },
            full_name: if g.full_name.is_empty() {
                key.clone()
            } else {
                g.full_name
            },
            deputy_count: g.deputy_count,
            deputies_with_network: g.deputies_with_network,
            total_cosignatures: g.total_cosignatures,
            in_group_count: g.in_group_count,
            out_group_count: g.out_group_count,
        })
        .collect();

    group_vec.sort_by(|a, b| {
        b.total_cosignatures
            .cmp(&a.total_cosignatures)
            .then(b.deputy_count.cmp(&a.deputy_count))
            .then(a.label.cmp(&b.label))
    });

    let index: HashMap<&str, usize> = group_vec
        .iter()
        .enumerate()
        .map(|(i, g)| (g.key.as_str(), i))
        .collect();

    let n = group_vec.len();
    let mut matrix = vec![vec![0u64; n]; n];

    for ((src, dst), v) in matrix_map {
        if let (Some(&i), Some(&j)) = (index.get(src.as_str()), index.get(dst.as_str())) {
            matrix[i][j] = matrix[i][j].saturating_add(v);
        }
    }

    let matrix_max = matrix
        .iter()
        .flat_map(|r| r.iter().copied())
        .max()
        .unwrap_or(0);

    let mut top_edges: Vec<GroupEdge> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let a_to_b = matrix[i][j];
            let b_to_a = matrix[j][i];
            let total = a_to_b.saturating_add(b_to_a);
            if total == 0 {
                continue;
            }
            top_edges.push(GroupEdge {
                a_key: group_vec[i].key.clone(),
                a_label: group_vec[i].label.clone(),
                b_key: group_vec[j].key.clone(),
                b_label: group_vec[j].label.clone(),
                a_to_b,
                b_to_a,
                total,
            });
        }
    }

    top_edges.sort_by(|a, b| {
        b.total
            .cmp(&a.total)
            .then(a.a_label.cmp(&b.a_label))
            .then(a.b_label.cmp(&b.b_label))
    });

    GroupNetworkSummary {
        groups: group_vec,
        matrix,
        matrix_max,
        total_deputes: stats.len(),
        deputies_with_network,
        deputies_without_network: stats.len().saturating_sub(deputies_with_network),
        total_cosignatures,
        total_in_group,
        total_out_group,
        top_edges,
        period_start,
        period_end,
    }
}

fn build_focus_group(
    summary: &GroupNetworkSummary,
    stats: &[DeputeStats],
    key: &str,
) -> Option<FocusGroupData> {
    let idx = summary.groups.iter().position(|g| g.key == key)?;
    let group = summary.groups[idx].clone();

    let row = summary.matrix.get(idx)?;
    let outgoing_total = row
        .iter()
        .enumerate()
        .filter(|(j, _)| *j != idx)
        .map(|(_, v)| *v)
        .sum::<u64>();
    let incoming_total = summary
        .matrix
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != idx)
        .map(|(_, r)| r[idx])
        .sum::<u64>();
    let self_total = summary.matrix[idx][idx];

    let outgoing_partners = build_partner_list_from_row(summary, idx, true);
    let incoming_partners = build_partner_list_from_row(summary, idx, false);
    let symmetric_partners = build_partner_list_symmetric(summary, idx);

    let mut bridge_deputies = stats
        .iter()
        .filter(|d| normalize_group_key(d.groupe_abrev.as_deref()) == key)
        .filter_map(|d| {
            let net = d.cosign_network.as_ref()?;
            let total = net.total_cosignatures as u64;
            if total == 0 {
                return None;
            }
            let out = net.out_group_count as u64;
            Some(BridgeDeputy {
                deputy_id: d.deputy_id.clone(),
                nom_complet: format!("{} {}", d.prenom, d.nom),
                out_group_count: out,
                total_cosignatures: total,
                transpartisan_rate: if total > 0 {
                    out as f64 / total as f64
                } else {
                    0.0
                },
                unique_cosignataires: net.unique_cosignataires,
            })
        })
        .collect::<Vec<_>>();

    bridge_deputies.sort_by(|a, b| {
        b.out_group_count
            .cmp(&a.out_group_count)
            .then_with(|| {
                b.transpartisan_rate
                    .partial_cmp(&a.transpartisan_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.nom_complet.cmp(&b.nom_complet))
    });

    Some(FocusGroupData {
        group,
        incoming_total,
        outgoing_total,
        self_total,
        incoming_partners,
        outgoing_partners,
        symmetric_partners,
        bridge_deputies,
    })
}

fn build_partner_list_from_row(
    summary: &GroupNetworkSummary,
    idx: usize,
    outgoing: bool,
) -> Vec<PartnerFlow> {
    let total = if outgoing {
        summary.matrix[idx]
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != idx)
            .map(|(_, v)| *v)
            .sum::<u64>()
    } else {
        summary
            .matrix
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != idx)
            .map(|(_, row)| row[idx])
            .sum::<u64>()
    };

    let mut rows = Vec::new();
    for j in 0..summary.groups.len() {
        if j == idx {
            continue;
        }
        let count = if outgoing {
            summary.matrix[idx][j]
        } else {
            summary.matrix[j][idx]
        };
        if count == 0 {
            continue;
        }
        let g = &summary.groups[j];
        rows.push(PartnerFlow {
            key: g.key.clone(),
            label: g.label.clone(),
            full_name: g.full_name.clone(),
            count,
            pct: if total > 0 {
                count as f64 * 100.0 / total as f64
            } else {
                0.0
            },
        });
    }
    rows.sort_by(|a, b| b.count.cmp(&a.count).then(a.label.cmp(&b.label)));
    rows
}

fn build_partner_list_symmetric(summary: &GroupNetworkSummary, idx: usize) -> Vec<PartnerFlow> {
    let mut total = 0u64;
    let mut rows = Vec::new();
    for j in 0..summary.groups.len() {
        if j == idx {
            continue;
        }
        let count = summary.matrix[idx][j].saturating_add(summary.matrix[j][idx]);
        if count == 0 {
            continue;
        }
        total = total.saturating_add(count);
        let g = &summary.groups[j];
        rows.push(PartnerFlow {
            key: g.key.clone(),
            label: g.label.clone(),
            full_name: g.full_name.clone(),
            count,
            pct: 0.0,
        });
    }
    for r in &mut rows {
        r.pct = if total > 0 {
            r.count as f64 * 100.0 / total as f64
        } else {
            0.0
        };
    }
    rows.sort_by(|a, b| b.count.cmp(&a.count).then(a.label.cmp(&b.label)));
    rows
}

fn apply_cosign_network(
    groups: &mut BTreeMap<String, TmpGroup>,
    matrix_map: &mut HashMap<(String, String), u64>,
    src_key: &str,
    net: &CosignNetworkStats,
) {
    if let Some(g) = groups.get_mut(src_key) {
        g.deputies_with_network += 1;
        g.total_cosignatures += net.total_cosignatures as u64;
        g.in_group_count += net.in_group_count as u64;
        g.out_group_count += net.out_group_count as u64;
    }

    let in_group = net.in_group_count as u64;
    if in_group > 0 {
        *matrix_map
            .entry((src_key.to_string(), src_key.to_string()))
            .or_insert(0) += in_group;
    }

    for bucket in &net.out_group_groups {
        let dst_key = normalize_group_key(bucket.groupe_abrev.as_deref());
        let dst_label =
            display_group_label(bucket.groupe_abrev.as_deref(), bucket.groupe_nom.as_deref());
        let dst_full = bucket
            .groupe_nom
            .clone()
            .unwrap_or_else(|| dst_label.clone());

        let entry = groups.entry(dst_key.clone()).or_default();
        if entry.label.is_empty() {
            entry.label = dst_label;
        }
        if entry.full_name.is_empty() {
            entry.full_name = dst_full;
        }

        let count = bucket.count_total as u64;
        if count > 0 {
            *matrix_map
                .entry((src_key.to_string(), dst_key))
                .or_insert(0) += count;
        }
    }
}

fn normalize_group_key(v: Option<&str>) -> String {
    let s = v.unwrap_or("NI").trim();
    if s.is_empty() {
        "NI".to_string()
    } else {
        s.to_uppercase()
    }
}

fn display_group_label(abrev: Option<&str>, nom: Option<&str>) -> String {
    let key = normalize_group_key(abrev);
    if key == "NI" {
        "NI".to_string()
    } else if !key.is_empty() {
        key
    } else {
        nom.unwrap_or("?").to_string()
    }
}

fn group_color(key: &str) -> &'static str {
    match key {
        "RN" => "#3b82f6",
        "EPR" | "RE" => "#f59e0b",
        "LFI" | "LFI-NFP" => "#ef4444",
        "SOC" | "PS" => "#ec4899",
        "HOR" => "#06b6d4",
        "GDR" => "#dc2626",
        "DEM" | "MODEM" | "MODEMDEM" | "DEMOCRATE" => "#a855f7",
        "LIOT" => "#84cc16",
        "UDR" | "UDDPLR" => "#f97316",
        "DR" => "#60a5fa",
        "ECOS" => "#22c55e",
        "NI" => "#94a3b8",
        _ => "#6b7280",
    }
}

fn fmt_u64(mut n: u64) -> String {
    if n < 1000 {
        return n.to_string();
    }
    let mut parts = Vec::new();
    while n > 0 {
        parts.push(format!("{:03}", n % 1000));
        n /= 1000;
    }
    if let Some(last) = parts.last_mut() {
        *last = last.trim_start_matches('0').to_string();
    }
    parts.reverse();
    parts.join(" ")
}

fn fmt_compact_u64(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}G", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn fmt_pct1(v: f64) -> String {
    format!("{:.1}%", v)
}

fn fmt_pct1_ratio(v: f64) -> String {
    format!("{:.1}%", v * 100.0)
}
