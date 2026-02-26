use leptos::*;
use leptos_router::use_query_map;
use wasm_bindgen::JsCast;

use crate::components::period_selector::PeriodSelector;
use crate::models::*;
use crate::store::use_store;
use crate::utils::{fmt_pct, groupe_color, matches_search};

fn period_from_query(raw: &str) -> Option<Period> {
    let s = raw.trim().to_ascii_lowercase();
    match s.as_str() {
        "p30" | "30" | "30j" | "30d" | "30days" => Some(Period::P30),
        "p180" | "180" | "180j" | "180d" | "180days" => Some(Period::P180),
        "leg" | "legislature" | "l17" => Some(Period::Leg),
        _ => None,
    }
}

#[component]
pub fn ComparerPage() -> impl IntoView {
    let store = use_store();
    let (period, set_period) = create_signal(Period::P180);
    let query = use_query_map();

    let (selected_a_id, set_selected_a_id) = create_signal::<Option<String>>(None);
    let (selected_b_id, set_selected_b_id) = create_signal::<Option<String>>(None);

    let raw_stats = create_memo(move |_| {
        store
            .stats_for(period.get())
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
    });

    let raw_stats_a = raw_stats.clone();
    let selected_a = create_memo(move |_| {
        let selected_id = selected_a_id.get();
        let stats = raw_stats_a.get();
        selected_id.and_then(|id| stats.into_iter().find(|d| d.deputy_id == id))
    });
    let raw_stats_b = raw_stats.clone();
    let selected_b = create_memo(move |_| {
        let selected_id = selected_b_id.get();
        let stats = raw_stats_b.get();
        selected_id.and_then(|id| stats.into_iter().find(|d| d.deputy_id == id))
    });

    create_effect(move |_| {
        query.with(|q| {
            if let Some(a) = q.get("a").filter(|s| !s.trim().is_empty()) {
                set_selected_a_id.set(Some(a.clone()));
            }
            if let Some(b) = q.get("b").filter(|s| !s.trim().is_empty()) {
                set_selected_b_id.set(Some(b.clone()));
            }
            if let Some(p) = q.get("period").and_then(|v| period_from_query(v)) {
                set_period.set(p);
            }
        });
    });

    view! {
        <div class="reveal">
            <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:1.5rem;flex-wrap:wrap;gap:1rem;">
                <div>
                    <h1 style="font-size:1.4rem;font-weight:700;margin:0 0 0.25rem 0;">"Comparer deux députés"</h1>
                    <p style="color:var(--text-muted);font-size:0.82rem;margin:0;">"Sélectionnez deux députés pour une comparaison côte à côte"</p>
                </div>
                <PeriodSelector period=period set_period=set_period />
            </div>

            <div class="cmp-select-grid" style="display:grid;grid-template-columns:1fr 1fr;gap:1.5rem;margin-bottom:2rem;">
                <Combobox
                    label="Député A"
                    color="var(--accent)"
                    all_stats=raw_stats
                    selected=selected_a
                    on_select=move |d| set_selected_a_id.set(Some(d.deputy_id.clone()))
                    on_clear=move || set_selected_a_id.set(None)
                />
                <Combobox
                    label="Député B"
                    color="#a78bfa"
                    all_stats=raw_stats
                    selected=selected_b
                    on_select=move |d| set_selected_b_id.set(Some(d.deputy_id.clone()))
                    on_clear=move || set_selected_b_id.set(None)
                />
            </div>

            {move || {
                let a = selected_a.get();
                let b = selected_b.get();
                if a.is_none() && b.is_none() {
                    return view! {
                        <div style="text-align:center;padding:3rem;color:var(--text-muted);border:1px dashed var(--bg-border);border-radius:8px;">
                            "Sélectionnez deux députés ci-dessus pour lancer la comparaison"
                        </div>
                    }.into_view();
                }
                view! {
                    <div class="cmp-table-wrap" style="background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:8px;overflow:hidden;overflow-x:auto;">
                        <table class="data-table" style="table-layout:fixed;">
                            <colgroup>
                                <col style="width:220px;"/>
                                <col/>
                                <col/>
                            </colgroup>
                            <thead>
                                <tr>
                                    <th>"Indicateur"</th>
                                    <th style="color:var(--accent);">
                                        {a.as_ref().map(|d| format!("{} {}", d.prenom, d.nom)).unwrap_or_else(|| "—".to_string())}
                                    </th>
                                    <th style="color:#a78bfa;">
                                        {b.as_ref().map(|d| format!("{} {}", d.prenom, d.nom)).unwrap_or_else(|| "—".to_string())}
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                <CmpRow label="Groupe"
                                    va=a.as_ref().map(|d| d.groupe_abrev.clone().unwrap_or_default())
                                    vb=b.as_ref().map(|d| d.groupe_abrev.clone().unwrap_or_default()) />
                                <CmpRow label="Département"
                                    va=a.as_ref().map(|d| d.dept.clone().unwrap_or_default())
                                    vb=b.as_ref().map(|d| d.dept.clone().unwrap_or_default()) />
                                <CmpRow label="Période de calcul"
                                    va=a.as_ref().map(|d| format!("{} → {}", d.period_start, d.period_end))
                                    vb=b.as_ref().map(|d| format!("{} → {}", d.period_start, d.period_end)) />
                                <CmpRow label="Scrutins éligibles"
                                    va=a.as_ref().map(|d| d.scrutins_eligibles.to_string())
                                    vb=b.as_ref().map(|d| d.scrutins_eligibles.to_string()) />
                                <CmpNum label="Participation"
                                    va=a.as_ref().map(|d| d.participation_rate)
                                    vb=b.as_ref().map(|d| d.participation_rate)
                                    fmt=fmt_pct higher_better=true />
                                <CmpNum label="Votes exprimés"
                                    va=a.as_ref().map(|d| d.votes_exprimes as f64)
                                    vb=b.as_ref().map(|d| d.votes_exprimes as f64)
                                    fmt=fmt_u32 higher_better=true />
                                <CmpRow label="Pour / Contre / Abst."
                                    va=a.as_ref().map(|d| format!("{} / {} / {}", d.pour_count, d.contre_count, d.abst_count))
                                    vb=b.as_ref().map(|d| format!("{} / {} / {}", d.pour_count, d.contre_count, d.abst_count)) />
                                <CmpNum label="Amendements déposés"
                                    va=a.as_ref().map(|d| d.amd_authored as f64)
                                    vb=b.as_ref().map(|d| d.amd_authored as f64)
                                    fmt=fmt_u32 higher_better=true />
                                <CmpNum label="Amendements adoptés"
                                    va=a.as_ref().map(|d| d.amd_adopted as f64)
                                    vb=b.as_ref().map(|d| d.amd_adopted as f64)
                                    fmt=fmt_u32 higher_better=true />
                                <CmpRow label="Taux d'adoption"
                                    va=a.as_ref().map(|d| d.amd_adoption_rate.map(|r| fmt_pct(r)).unwrap_or_else(|| "—".to_string()))
                                    vb=b.as_ref().map(|d| d.amd_adoption_rate.map(|r| fmt_pct(r)).unwrap_or_else(|| "—".to_string())) />
                                <CmpNum label="Cosignatures"
                                    va=a.as_ref().map(|d| d.amd_cosigned as f64)
                                    vb=b.as_ref().map(|d| d.amd_cosigned as f64)
                                    fmt=fmt_u32 higher_better=true />
                            </tbody>
                        </table>
                    </div>
                }.into_view()
            }}
        </div>
    }
}

/// Combobox accessible avec ARIA complet
#[component]
fn Combobox<FS, FC>(
    label: &'static str,
    color: &'static str,
    all_stats: Memo<Vec<DeputeStats>>,
    selected: Memo<Option<DeputeStats>>,
    on_select: FS,
    on_clear: FC,
) -> impl IntoView
where
    FS: Fn(DeputeStats) + 'static + Clone,
    FC: Fn() + 'static + Clone,
{
    let (query, set_query) = create_signal(String::new());
    let (open, set_open) = create_signal(false);
    let list_id = label.replace(' ', "-").to_lowercase();
    let input_id = format!("{list_id}-input");

    let suggestions = create_memo(move |_| {
        let q = query.get();
        if q.trim().is_empty() {
            return vec![];
        }
        all_stats
            .get()
            .into_iter()
            .filter(|d| matches_search(&format!("{} {}", d.prenom, d.nom), &q))
            .take(8)
            .collect::<Vec<_>>()
    });

    view! {
        <div style="position:relative;">
            <label
                for=input_id.clone()
                style=format!("display:block;font-size:0.75rem;font-weight:600;color:{color};margin-bottom:0.4rem;text-transform:uppercase;letter-spacing:0.06em;")>
                {label}
            </label>

            {move || selected.get().map(|d| {
                let grp_color = groupe_color(d.groupe_abrev.as_deref());
                let on_clear2 = on_clear.clone();
                view! {
                    <div style="display:flex;align-items:center;justify-content:space-between;padding:0.5rem 0.75rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:6px;margin-bottom:0.25rem;">
                        <div>
                            <span style="font-weight:500;">{format!("{} {}", d.prenom, d.nom)}</span>
                            {d.groupe_abrev.as_ref().map(|g| view! {
                                <span class="badge" style=format!("margin-left:0.5rem;border-color:{grp_color};color:{grp_color};")>
                                    {g.clone()}
                                </span>
                            })}
                        </div>
                        <button
                            on:click=move |_| { on_clear2(); }
                            style="background:none;border:none;cursor:pointer;color:var(--text-muted);font-size:1rem;padding:0;"
                            aria-label="Désélectionner ce député"
                        >"×"</button>
                    </div>
                }
            })}

            {move || {
                if selected.get().is_some() {
                    return None;
                }

                let on_select2 = on_select.clone();
                let input_id2 = input_id.clone();
                let list_id2 = list_id.clone();

                Some(view! {
                    <div>
                        <input
                            id=input_id2.clone()
                            type="text"
                            placeholder="Rechercher un député…"
                            prop:value=move || query.get()
                            on:input=move |e| {
                                set_query.set(event_target_value(&e));
                                set_open.set(true);
                            }
                            on:focus=move |_| set_open.set(true)
                            on:keydown=move |e: web_sys::KeyboardEvent| {
                                if e.key() == "Escape" { set_open.set(false); }
                            }
                            role="combobox"
                            aria-expanded=move || open.get().to_string()
                            aria-controls=list_id2.clone()
                            aria-autocomplete="list"
                            autocomplete="off"
                        />

                        {move || {
                            let items = suggestions.get();
                            if !open.get() || items.is_empty() {
                                return None;
                            }

                            let on_select3 = on_select2.clone();
                            let list_id3 = list_id2.clone();

                            Some(view! {
                                // Overlay pour fermer au clic dehors
                                <div
                                    style="position:fixed;inset:0;z-index:40;"
                                    on:click=move |_| set_open.set(false)
                                ></div>
                                <ul
                                    id=list_id3
                                    role="listbox"
                                    style="position:absolute;top:100%;left:0;right:0;border:1px solid var(--bg-border);border-top:none;border-radius:0 0 6px 6px;background:var(--bg-secondary);max-height:220px;overflow-y:auto;margin:0;padding:0;list-style:none;z-index:50;"
                                >
                                    {items.into_iter().map(|d| {
                                        let on_select4 = on_select3.clone();
                                        let label_str = format!("{} {} — {}",
                                            d.prenom, d.nom,
                                            d.groupe_abrev.as_deref().unwrap_or("?"));
                                        let d_clone = d.clone();
                                        view! {
                                            <li
                                                role="option"
                                                style="padding:0.5rem 0.75rem;cursor:pointer;font-size:0.82rem;border-bottom:1px solid var(--bg-border);"
                                                on:click=move |e| {
                                                    e.stop_propagation(); // ne pas fermer via l'overlay
                                                    on_select4(d_clone.clone());
                                                    set_open.set(false);
                                                    set_query.set(String::new());
                                                }
                                                on:mouseover=move |e| {
                                                    if let Some(el) = e.target()
                                                        .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok()) {
                                                        let _ = el.style().set_property("background", "var(--accent-dim)");
                                                    }
                                                }
                                                on:mouseout=move |e| {
                                                    if let Some(el) = e.target()
                                                        .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok()) {
                                                        let _ = el.style().remove_property("background");
                                                    }
                                                }
                                            >
                                                {label_str}
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            })
                        }}
                    </div>
                })
            }}
        </div>
    }
}

fn fmt_u32(v: f64) -> String {
    format!("{}", v as u32)
}

#[component]
fn CmpRow(label: &'static str, va: Option<String>, vb: Option<String>) -> impl IntoView {
    view! {
        <tr>
            <td style="color:var(--text-muted);font-size:0.78rem;">{label}</td>
            <td style="color:var(--accent);">{va.unwrap_or_else(|| "—".to_string())}</td>
            <td style="color:#a78bfa;">{vb.unwrap_or_else(|| "—".to_string())}</td>
        </tr>
    }
}

#[component]
fn CmpNum(
    label: &'static str,
    va: Option<f64>,
    vb: Option<f64>,
    fmt: fn(f64) -> String,
    higher_better: bool,
) -> impl IntoView {
    let winner = match (va, vb) {
        (Some(a), Some(b)) if (a - b).abs() > 1e-9 => {
            if higher_better {
                if a > b {
                    Some('a')
                } else {
                    Some('b')
                }
            } else {
                if a < b {
                    Some('a')
                } else {
                    Some('b')
                }
            }
        }
        _ => None,
    };
    let fmt_b = fmt;
    view! {
        <tr>
            <td style="color:var(--text-muted);font-size:0.78rem;">{label}</td>
            <td style=move || if winner == Some('a') {
                "color:var(--accent);font-weight:700;"
            } else { "color:var(--accent);" }>
                {va.map(|v| fmt(v)).unwrap_or_else(|| "—".to_string())}
                {if winner == Some('a') { " ●" } else { "" }}
            </td>
            <td style=move || if winner == Some('b') {
                "color:#a78bfa;font-weight:700;"
            } else { "color:#a78bfa;" }>
                {vb.map(|v| fmt_b(v)).unwrap_or_else(|| "—".to_string())}
                {if winner == Some('b') { " ●" } else { "" }}
            </td>
        </tr>
    }
}
