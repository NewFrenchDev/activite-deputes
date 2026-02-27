use leptos::*;
use leptos_router::A;
use std::collections::{HashMap, HashSet};

use crate::api::{fetch_amendements_index, fetch_amendements_month, fetch_deputes, fetch_dossiers_min};
use crate::models::{AmendementEvent, AmendementsIndex, AmendementsMonthFile, DeputeInfo, DossiersMin};
use crate::utils::{app_href, groupe_color, matches_search};

fn fmt_date_fr(iso: &str) -> String {
    // "YYYY-MM-DD" -> "DD/MM/YYYY" (fallback si format inattendu)
    let parts: Vec<&str> = iso.split('-').collect();
    if parts.len() == 3 {
        format!("{}/{}/{}", parts[2], parts[1], parts[0])
    } else {
        iso.to_string()
    }
}

fn type_label(t: &str) -> String {
    match t {
        "DEPOT" => "Dépôt".to_string(),
        "EXAMEN" => "Examen".to_string(),
        "SORT" => "Sort".to_string(),
        "CIRCULATION" => "Circulation".to_string(),
        other => other.to_string(),
    }
}

fn type_class(t: &str) -> &'static str {
    match t {
        "DEPOT" => "amd-typechip--depot",
        "EXAMEN" => "amd-typechip--examen",
        "SORT" => "amd-typechip--sort",
        "CIRCULATION" => "amd-typechip--circ",
        _ => "amd-typechip--circ",
    }
}

fn day_summary(events: &[AmendementEvent]) -> (usize, usize, usize) {
    let evts = events.len();
    let mut amds: HashSet<&str> = HashSet::new();
    let mut authors: HashSet<&str> = HashSet::new();
    for e in events {
        amds.insert(e.id.as_str());
        if let Some(a) = e.aid.as_deref() {
            if !a.is_empty() {
                authors.insert(a);
            }
        }
    }
    (evts, amds.len(), authors.len())
}

fn compute_kpis(events: &[AmendementEvent]) -> HashMap<&'static str, usize> {
    let mut out: HashMap<&'static str, usize> = HashMap::new();
    out.insert("events", events.len());

    let mut amds: HashSet<&str> = HashSet::new();
    let mut authors: HashSet<&str> = HashSet::new();

    let mut depots = 0usize;
    let mut examens = 0usize;
    let mut sorts = 0usize;
    let mut adopted = 0usize;

    for e in events {
        amds.insert(e.id.as_str());
        if let Some(a) = e.aid.as_deref() {
            if !a.is_empty() {
                authors.insert(a);
            }
        }

        match e.t.as_str() {
            "DEPOT" => depots += 1,
            "EXAMEN" => examens += 1,
            "SORT" => {
                sorts += 1;
                if e.ok {
                    adopted += 1;
                }
            }
            _ => {}
        }
    }

    out.insert("amds", amds.len());
    out.insert("authors", authors.len());
    out.insert("depots", depots);
    out.insert("examens", examens);
    out.insert("sorts", sorts);
    out.insert("adopted", adopted);
    out
}

fn top_deputies(events: &[AmendementEvent]) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for e in events {
        if let Some(a) = e.aid.as_ref() {
            if !a.is_empty() {
                *counts.entry(a.clone()).or_insert(0) += 1;
            }
        }
    }
    let mut v: Vec<(String, usize)> = counts.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v.truncate(10);
    v
}

fn deputy_display(dep: Option<&DeputeInfo>) -> (String, Option<String>, Option<String>) {
    match dep {
        Some(d) => (
            format!("{} {}", d.prenom, d.nom),
            d.groupe_abrev.clone(),
            Some(d.id.clone()),
        ),
        None => ("—".to_string(), None, None),
    }
}

#[component]
pub fn AmendementsPage() -> impl IntoView {
    let (selected_month, set_selected_month) = create_signal::<Option<String>>(None);
    let (selected_day, set_selected_day) = create_signal::<Option<String>>(None);
    let (filter, set_filter) = create_signal(String::new());
    let (show_undated, set_show_undated) = create_signal(false);

    // Données de base
    let index_res = create_resource(|| (), |_| async move { fetch_amendements_index().await });
    let deputes_res = create_resource(|| (), |_| async move { fetch_deputes().await });
    let dossiers_res = create_resource(|| (), |_| async move { fetch_dossiers_min().await });

    // Shard du mois sélectionné
    let month_res = create_resource(
        move || selected_month.get(),
        |m| async move {
            if let Some(m) = m {
                fetch_amendements_month(&m).await.map(Some)
            } else {
                Ok(None)
            }
        },
    );

    // Initialiser le mois par défaut quand index.json arrive
    create_effect(move |_| {
        if selected_month.get().is_some() {
            return;
        }
        if let Some(Ok(idx)) = index_res.get() {
            if let Some(first) = idx.months.first() {
                set_selected_month.set(Some(first.month.clone()));
            }
        }
    });

    // Initialiser le jour par défaut quand le mois est chargé
    create_effect(move |_| {
        let day = selected_day.get();
        if let Some(Ok(Some(mf))) = month_res.get() {
            let mut keys: Vec<String> = mf.days.keys().cloned().collect();
            keys.sort_by(|a, b| b.cmp(a));
            let best = keys.first().cloned();
            if best.is_some() {
                // si pas de jour sélectionné ou jour invalide, choisir le plus récent
                let invalid = day.as_ref().map(|d| !mf.days.contains_key(d)).unwrap_or(true);
                if invalid {
                    set_selected_day.set(best);
                }
            }
        }
    });

    // Maps (députés / dossiers)
    let deputes_map = create_memo(move |_| {
        deputes_res
            .get()
            .and_then(|r| r.ok())
            .map(|list| list.into_iter().map(|d| (d.id.clone(), d)).collect::<HashMap<_, _>>())
            .unwrap_or_default()
    });

    let dossiers_map = create_memo(move |_| {
        dossiers_res.get().and_then(|r| r.ok()).unwrap_or_default()
    });

    view! {
        <div style="max-width:1400px;margin:0 auto;padding:1.5rem;">
            <div style="display:flex;align-items:flex-start;justify-content:space-between;gap:1rem;flex-wrap:wrap;margin-bottom:1rem;">
                <div>
                    <h1 style="margin:0;font-size:1.65rem;letter-spacing:-0.02em;">"Calendrier des amendements"</h1>
                    <p style="margin:0.35rem 0 0 0;color:var(--text-secondary);max-width:78ch;line-height:1.35;">
                        "Objectif : rendre l’activité amendements lisible, jour par jour. Chaque ligne correspond à un "
                        <b>"évènement"</b>
                        " (dépôt / examen / sort / circulation) issu du cycle de vie open data."
                    </p>
                </div>

                <div class="amd-controls" style="display:flex;align-items:center;gap:0.6rem;flex-wrap:wrap;min-width:320px;">
                    <div style="min-width:220px;">
                        <div class="amd-label">"Mois"</div>
                        <select
                            on:change=move |ev| {
                                let v = event_target_value(&ev);
                                set_selected_month.set(Some(v));
                                set_filter.set(String::new());
                            }
                        >
                            {move || {
                                match index_res.get() {
                                    Some(Ok(idx)) => idx.months.iter().map(|m| {
                                        let val = m.month.clone();
                                        let selected = selected_month.get().as_deref() == Some(val.as_str());
                                        view!{ <option value=val.clone() selected=selected>{val}</option> }
                                    }).collect_view(),
                                    _ => view! { <option value="">"—"</option> }.into_view(),
                                }
                            }}
                        </select>
                    </div>

                    <div style="min-width:190px;">
                        <div class="amd-label">"Date"</div>
                        <input
                            type="date"
                            prop:value=move || selected_day.get().unwrap_or_default()
                            on:change=move |ev| {
                                let v = event_target_value(&ev);
                                if v.len() >= 7 {
                                    // si l'utilisateur choisit un jour qui change de mois, basculer aussi
                                    let m = v[..7].to_string();
                                    set_selected_month.set(Some(m));
                                }
                                set_selected_day.set(Some(v));
                                set_filter.set(String::new());
                            }
                            style="padding:0.5rem 0.75rem;border:1px solid var(--bg-border);border-radius:6px;background:var(--bg-secondary);color:var(--text-primary);font-size:0.86rem;font-weight:500;"
                        />
                    </div>

                    <button class="btn" on:click=move |_| {
                        // "Aujourd'hui" : pour la démo, on se cale sur le mois le plus récent + jour le plus récent.
                        if let Some(Ok(idx)) = index_res.get() {
                            if let Some(first) = idx.months.first() {
                                set_selected_month.set(Some(first.month.clone()));
                                set_filter.set(String::new());
                            }
                        }
                    }>
                        "Aujourd’hui"
                    </button>
                </div>
            </div>

            <div class="kpi-card" style="margin-bottom:0.75rem;">
                <details open>
                    <summary style="cursor:pointer;display:flex;align-items:center;justify-content:space-between;gap:0.75rem;">
                        <div style="display:flex;align-items:center;gap:0.6rem;flex-wrap:wrap;">
                            <span class="badge">"Comprendre ces données"</span>
                            <span style="color:var(--text-muted);font-size:0.85rem;">"Glossaire + limites (open data)"</span>
                        </div>
                        <span style="color:var(--text-muted);">"▾"</span>
                    </summary>
                    <hr style="border:0;border-top:1px solid var(--bg-border);margin:0.8rem 0;" />
                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;">
                        <div>
                            <div class="amd-label">"Glossaire"</div>
                            <div style="margin-top:0.35rem;color:var(--text-secondary);line-height:1.35;">
                                <div style="display:flex;gap:0.5rem;align-items:center;margin:0.25rem 0;">
                                    <span class="amd-typechip amd-typechip--depot">"DEPOT"</span>
                                    <span>"un amendement est déposé (naissance)."</span>
                                </div>
                                <div style="display:flex;gap:0.5rem;align-items:center;margin:0.25rem 0;">
                                    <span class="amd-typechip amd-typechip--examen">"EXAMEN"</span>
                                    <span>"étape de discussion (commission / séance), si renseignée."</span>
                                </div>
                                <div style="display:flex;gap:0.5rem;align-items:center;margin:0.25rem 0;">
                                    <span class="amd-typechip amd-typechip--sort">"SORT"</span>
                                    <span>"résultat connu : adopté / rejeté / retiré / tombé…"</span>
                                </div>
                                <div style="display:flex;gap:0.5rem;align-items:center;margin:0.25rem 0;">
                                    <span class="amd-typechip amd-typechip--circ">"CIRCULATION"</span>
                                    <span>"étape intermédiaire (souvent incomplète selon les dossiers)."</span>
                                </div>
                            </div>
                        </div>
                        <div>
                            <div class="amd-label">"À savoir"</div>
                            <ul style="margin:0.35rem 0 0;padding-left:1.2rem;color:var(--text-secondary);line-height:1.35;">
                                <li>"Un amendement peut produire plusieurs évènements datés (ex : dépôt + sort)."</li>
                                <li>"Les dates ne sont pas toujours présentes : certains amendements finissent dans “sans date”."</li>
                                <li>"Ce calendrier montre “ce qui se passe ce jour-là”, pas le texte intégral."</li>
                            </ul>
                        </div>
                    </div>
                </details>
            </div>

            <div class="amd-main-grid" style="display:grid;grid-template-columns: 380px 1fr;gap:0.75rem;align-items:start;">
                // Colonne gauche (jours)
                <div class="kpi-card" style="min-width:300px;">
                    <div style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;flex-wrap:wrap;margin-bottom:0.6rem;">
                        <h2 style="margin:0;font-size:1.02rem;">"Jours du mois"</h2>
                        <span style="color:var(--text-muted);font-size:0.78rem;">
                            {move || match month_res.get() {
                                Some(Ok(Some(mf))) => format!("{} jours", mf.days.len()),
                                _ => "—".to_string(),
                            }}
                        </span>
                    </div>

                    <div class="amd-daylist">
                        {move || {
                            match month_res.get() {
                                Some(Ok(Some(mf))) => {
                                    let mut days: Vec<String> = mf.days.keys().cloned().collect();
                                    days.sort_by(|a, b| b.cmp(a));
                                    days.into_iter().map(|d| {
                                        let events = mf.days.get(&d).cloned().unwrap_or_default();
                                        let (evts, amds, authors) = day_summary(&events);
                                        let active = selected_day.get().as_deref() == Some(d.as_str());
                                        let date_fr = fmt_date_fr(&d);
                                        view! {
                                            <button
                                                type="button"
                                                class=move || {
                                                    if active { "amd-daybtn active" } else { "amd-daybtn" }
                                                }
                                                on:click=move |_| {
                                                    set_selected_day.set(Some(d.clone()));
                                                    set_filter.set(String::new());
                                                }
                                            >
                                                <div class="amd-daymeta">
                                                    <div class="amd-daydate">{date_fr}</div>
                                                    <div class="amd-daymini">{format!("{} amd • {} auteurs", amds, authors)}</div>
                                                </div>
                                                <div class="amd-daycount">{format!("{} évt", evts)}</div>
                                            </button>
                                        }
                                    }).collect_view()
                                }
                                Some(Ok(None)) => view! { <p style="margin:0;color:var(--text-muted);">"Choisis un mois."</p> }.into_view(),
                                Some(Err(e)) => view! { <p style="margin:0;color:var(--danger);">{e}</p> }.into_view(),
                                None => view! { <p style="margin:0;color:var(--text-muted);">"Chargement…"</p> }.into_view(),
                            }
                        }}
                    </div>

                    <hr style="border:0;border-top:1px solid var(--bg-border);margin:0.8rem 0;" />

                    <div class="amd-toggle">
                        <div>
                            <div style="font-weight:750;">"Inclure “sans date”"</div>
                            <div style="color:var(--text-muted);font-size:0.78rem;">"Utile pour compléter l’historique (non daté)."</div>
                        </div>
                        <input
                            type="checkbox"
                            prop:checked=move || show_undated.get()
                            on:change=move |ev| set_show_undated.set(event_target_checked(&ev))
                        />
                    </div>
                    <div style="margin-top:0.6rem;color:var(--text-muted);font-size:0.78rem;line-height:1.35;">
                        {move || match index_res.get() {
                            Some(Ok(idx)) => format!("Sans date : {} (fichier: {})", idx.undated_count, idx.undated_file),
                            _ => "Sans date : —".to_string(),
                        }}
                    </div>
                </div>

                // Colonne droite (détail du jour)
                <div class="kpi-card" style="min-width:340px;">
                    <div style="display:flex;align-items:flex-start;justify-content:space-between;gap:0.75rem;flex-wrap:wrap;margin-bottom:0.6rem;">
                        <div>
                            <h2 style="margin:0;font-size:1.02rem;">"Détail du jour"</h2>
                            <div style="color:var(--text-muted);font-size:0.78rem;">
                                {move || selected_day.get().map(|d| fmt_date_fr(&d)).unwrap_or_else(|| "—".to_string())}
                            </div>
                        </div>
                        <div style="min-width:280px;max-width:420px;flex:1;">
                            <div class="amd-label">"Filtrer (député, dossier, n°, id…)"</div>
                            <input
                                type="text"
                                placeholder="ex: adopté, PLF, Durand, 123..."
                                prop:value=move || filter.get()
                                on:input=move |ev| set_filter.set(event_target_value(&ev))
                                style="width:100%;"
                            />
                        </div>
                    </div>

                    {move || {
                        let mf = match month_res.get() {
                            Some(Ok(Some(mf))) => mf,
                            Some(Ok(None)) => return view! { <p style="margin:0;color:var(--text-muted);">"Choisis un mois."</p> }.into_view(),
                            Some(Err(e)) => return view! { <p style="margin:0;color:var(--danger);">{e}</p> }.into_view(),
                            None => return view! { <p style="margin:0;color:var(--text-muted);">"Chargement…"</p> }.into_view(),
                        };
                        let day = match selected_day.get() {
                            Some(d) => d,
                            None => return view! { <p style="margin:0;color:var(--text-muted);">"Choisis un jour."</p> }.into_view(),
                        };
                        let events = mf.days.get(&day).cloned().unwrap_or_default();
                        let kpis = compute_kpis(&events);
                        let top = top_deputies(&events);
                        let dep_map = deputes_map.get();
                        let dos_map = dossiers_map.get();

                        // Préparer liste filtrée (table)
                        let needle = filter.get();
                        let filtered: Vec<AmendementEvent> = if needle.trim().is_empty() {
                            events.clone()
                        } else {
                            events
                                .iter()
                                .cloned()
                                .filter(|e| {
                                    let dep_name = e
                                        .aid
                                        .as_deref()
                                        .and_then(|id| dep_map.get(id))
                                        .map(|d| format!("{} {} {}", d.prenom, d.nom, d.groupe_abrev.clone().unwrap_or_default()))
                                        .unwrap_or_default();
                                    let dossier_title = e.did.as_deref().and_then(|id| dos_map.get(id)).cloned().unwrap_or_default();
                                    let hay = format!(
                                        "{} {} {} {} {} {} {} {} {}", 
                                        e.t, e.id, e.n.clone().unwrap_or_default(), dep_name, e.did.clone().unwrap_or_default(), dossier_title, e.s.clone().unwrap_or_default(), e.exp.clone().unwrap_or_default(), e.mis.clone().unwrap_or_default()
                                    );
                                    matches_search(&hay, &needle)
                                })
                                .collect()
                        };

                        view! {
                            <div class="amd-kpi-row">
                                <div class="amd-kpi-mini">
                                    <div class="amd-kmini-l">"Évènements"</div>
                                    <div class="amd-kmini-v">{kpis.get("events").copied().unwrap_or(0)}</div>
                                </div>
                                <div class="amd-kpi-mini">
                                    <div class="amd-kmini-l">"Amendements"</div>
                                    <div class="amd-kmini-v">{kpis.get("amds").copied().unwrap_or(0)}</div>
                                </div>
                                <div class="amd-kpi-mini">
                                    <div class="amd-kmini-l">"Auteurs"</div>
                                    <div class="amd-kmini-v">{kpis.get("authors").copied().unwrap_or(0)}</div>
                                </div>
                                <div class="amd-kpi-mini">
                                    <div class="amd-kmini-l">"Sorts"</div>
                                    <div class="amd-kmini-v">{kpis.get("sorts").copied().unwrap_or(0)}</div>
                                    <div style="color:var(--text-muted);font-size:0.78rem;margin-top:0.15rem;">
                                        {format!("Adoptés {}", kpis.get("adopted").copied().unwrap_or(0))}
                                    </div>
                                </div>
                                <div class="amd-kpi-mini">
                                    <div class="amd-kmini-l">"Dépôts"</div>
                                    <div class="amd-kmini-v">{kpis.get("depots").copied().unwrap_or(0)}</div>
                                </div>
                                <div class="amd-kpi-mini">
                                    <div class="amd-kmini-l">"Examens"</div>
                                    <div class="amd-kmini-v">{kpis.get("examens").copied().unwrap_or(0)}</div>
                                </div>
                            </div>

                            <div class="amd-sidecards">
                                <div class="kpi-card" style="padding:0.9rem 1rem;">
                                    <div style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;flex-wrap:wrap;margin-bottom:0.35rem;">
                                        <h3 style="margin:0;font-size:0.98rem;">"Députés les + actifs"</h3>
                                        <span style="color:var(--text-muted);font-size:0.78rem;">{format!("{} auteurs", top.len())}</span>
                                    </div>
                                    <div class="amd-rank">
                                        {top.into_iter().map(|(id, count)| {
                                            let dep = dep_map.get(&id);
                                            let (name, grp, href_id) = deputy_display(dep);
                                            let dot = groupe_color(grp.as_deref());
                                            let href = href_id.map(|x| app_href(&format!("/depute/{x}")));
                                            view!{
                                                <div class="amd-rankitem">
                                                    <div class="amd-rankname">
                                                        <span class="amd-dot" style=format!("background:{dot};")></span>
                                                        <div style="min-width:0;">
                                                            {match href {
                                                                Some(h) => view!{ <A href=h class="amd-ellipsis amd-link">{name}</A> }.into_view(),
                                                                None => view!{ <span class="amd-ellipsis" style="font-weight:800;">{name}</span> }.into_view(),
                                                            }}
                                                            <div style="margin-top:0.15rem;">
                                                                {grp.clone().map(|g| view!{ <span class="badge">{g}</span> })}
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div class="amd-rankcount">{count}</div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                <div class="kpi-card" style="padding:0.9rem 1rem;">
                                    <div style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;flex-wrap:wrap;margin-bottom:0.35rem;">
                                        <h3 style="margin:0;font-size:0.98rem;">"Lecture rapide"</h3>
                                        <span style="color:var(--text-muted);font-size:0.78rem;">"Exposés des amendements"</span>
                                    </div>
                                    <div style="max-height:400px;overflow-y:auto;">
                                        {events.iter().filter(|e| e.exp.is_some()).take(10).map(|e| {
                                            let dep = e.aid.as_deref().and_then(|id| dep_map.get(id));
                                            let (name, grp, _) = deputy_display(dep);
                                            let dot = groupe_color(grp.as_deref());
                                            let expose = e.exp.clone().unwrap_or_default();
                                            let type_chip = type_class(&e.t);
                                            view! {
                                                <div style=format!("margin:0.6rem 0;padding:0.65rem;border-left:3px solid {};background:rgba(255,255,255,.02);border-radius:4px;", dot)>
                                                    <div style="display:flex;align-items:center;gap:0.5rem;margin-bottom:0.35rem;flex-wrap:wrap;">
                                                        <span class={format!("amd-typechip {}", type_chip)} style="font-size:0.7rem;">{type_label(&e.t)}</span>
                                                        <span style="font-weight:700;font-size:0.84rem;">{name}</span>
                                                        {grp.map(|g| view!{ <span class="badge" style="font-size:0.7rem;">{g}</span> })}
                                                    </div>
                                                    <div style="color:var(--text-secondary);font-size:0.84rem;line-height:1.4;">
                                                        {if expose.chars().count() > 200 {
                                                            format!("{}...", expose.chars().take(200).collect::<String>())
                                                        } else {
                                                            expose
                                                        }}
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                        {if events.iter().filter(|e| e.exp.is_some()).count() == 0 {
                                            view! {
                                                <div style="color:var(--text-muted);font-size:0.86rem;padding:0.75rem 0;">
                                                    <p style="margin:0.35rem 0;">
                                                        <span class="badge">"Info"</span>
                                                        " Aucun exposé sommaire disponible pour ce jour."
                                                    </p>
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {}.into_view()
                                        }}
                                    </div>
                                </div>
                            </div>

                            <hr style="border:0;border-top:1px solid var(--bg-border);margin:0.85rem 0;" />

                            <div style="display:flex;align-items:center;justify-content:space-between;gap:0.75rem;flex-wrap:wrap;margin-bottom:0.6rem;">
                                <h3 style="margin:0;font-size:1rem;">"Journal des évènements"</h3>
                                <span style="color:var(--text-muted);font-size:0.78rem;">
                                    {format!("{} évènements • {} affichés", events.len(), filtered.len())}
                                </span>
                            </div>

                            <div class="amd-table-wrap" role="region" aria-label="Tableau évènements">
                                <table style="border-collapse:collapse;width:100%;min-width:1200px;background:rgba(255,255,255,.02);">
                                    <thead>
                                        <tr>
                                            <th style="position:sticky;top:0;background:rgba(31,41,55,.95);color:var(--text-secondary);font-size:.72rem;font-weight:800;letter-spacing:.06em;text-transform:uppercase;padding:.65rem .75rem;text-align:left;border-bottom:1px solid var(--bg-border);white-space:nowrap;">"Type"</th>
                                            <th style="position:sticky;top:0;background:rgba(31,41,55,.95);color:var(--text-secondary);font-size:.72rem;font-weight:800;letter-spacing:.06em;text-transform:uppercase;padding:.65rem .75rem;text-align:left;border-bottom:1px solid var(--bg-border);white-space:nowrap;">"Auteur"</th>
                                            <th style="position:sticky;top:0;background:rgba(31,41,55,.95);color:var(--text-secondary);font-size:.72rem;font-weight:800;letter-spacing:.06em;text-transform:uppercase;padding:.65rem .75rem;text-align:left;border-bottom:1px solid var(--bg-border);white-space:nowrap;">"Amendement"</th>
                                            <th style="position:sticky;top:0;background:rgba(31,41,55,.95);color:var(--text-secondary);font-size:.72rem;font-weight:800;letter-spacing:.06em;text-transform:uppercase;padding:.65rem .75rem;text-align:left;border-bottom:1px solid var(--bg-border);white-space:nowrap;">"Dossier / Mission"</th>
                                            <th style="position:sticky;top:0;background:rgba(31,41,55,.95);color:var(--text-secondary);font-size:.72rem;font-weight:800;letter-spacing:.06em;text-transform:uppercase;padding:.65rem .75rem;text-align:left;border-bottom:1px solid var(--bg-border);white-space:nowrap;">"Article"</th>
                                            <th style="position:sticky;top:0;background:rgba(31,41,55,.95);color:var(--text-secondary);font-size:.72rem;font-weight:800;letter-spacing:.06em;text-transform:uppercase;padding:.65rem .75rem;text-align:left;border-bottom:1px solid var(--bg-border);white-space:nowrap;">"Sort"</th>
                                            <th style="position:sticky;top:0;background:rgba(31,41,55,.95);color:var(--text-secondary);font-size:.72rem;font-weight:800;letter-spacing:.06em;text-transform:uppercase;padding:.65rem .75rem;text-align:left;border-bottom:1px solid var(--bg-border);white-space:nowrap;">"Exposé sommaire"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {filtered.into_iter().enumerate().map(|(_idx, e)| {
                                            let dep = e.aid.as_deref().and_then(|id| dep_map.get(id));
                                            let (name, grp, href_id) = deputy_display(dep);
                                            let dot = groupe_color(grp.as_deref());
                                            let href = href_id.map(|x| app_href(&format!("/depute/{x}")));

                                            let dossier_title = e.did.as_deref().and_then(|id| dos_map.get(id)).cloned().unwrap_or_else(|| "—".to_string());
                                            let dossier_id = e.did.clone().unwrap_or_default();

                                            let amd_label = e.n.clone().unwrap_or_else(|| "".to_string());
                                            let sort_view = match e.s.clone() {
                                                Some(s) if e.ok => view!{ <span class="badge" style="border-color:rgba(52,211,153,.35);background:rgba(52,211,153,.12);color:var(--success);">{s}</span> }.into_view(),
                                                Some(s) => view!{ <span class="badge">{s}</span> }.into_view(),
                                                None => view!{ <span style="color:var(--text-muted);">"—"</span> }.into_view(),
                                            };

                                            let (show_cosig, set_show_cosig) = create_signal(false);
                                            let cosig_count = e.cos.len();
                                            let has_cosig = cosig_count > 0;

                                            let cosig_list = e.cos.clone();
                                            let auteur_type = e.aty.clone();
                                            let mission = e.mis.clone();
                                            let expose = e.exp.clone();

                                            // Clone dep_map for use in the reactive closure
                                            let dep_map_clone = dep_map.clone();

                                            view!{
                                                <tr>
                                                    <td style="padding:.62rem .75rem;border-bottom:1px solid rgba(255,255,255,.06);font-size:.86rem;vertical-align:middle;">
                                                        <span class={format!("amd-typechip {}", type_class(&e.t))}>{type_label(&e.t)}</span>
                                                    </td>
                                                    <td style="padding:.62rem .75rem;border-bottom:1px solid rgba(255,255,255,.06);font-size:.86rem;vertical-align:middle;">
                                                        <div style="display:flex;align-items:center;gap:.45rem;min-width:0;">
                                                            <span class="amd-dot" style=format!("background:{dot};")></span>
                                                            <div style="min-width:0;width:100%;">
                                                                <div style="display:flex;align-items:center;gap:0.35rem;flex-wrap:wrap;">
                                                                    {match href {
                                                                        Some(h) => view!{ <A href=h class="amd-ellipsis amd-link">{name}</A> }.into_view(),
                                                                        None => view!{ <span class="amd-ellipsis" style="font-weight:800;">{name}</span> }.into_view(),
                                                                    }}
                                                                    {auteur_type.as_ref().map(|at| view!{
                                                                        <span style="color:var(--text-muted);font-size:0.75rem;">{format!("({})", at)}</span>
                                                                    })}
                                                                </div>
                                                                <div style="margin-top:0.12rem;display:flex;align-items:center;gap:0.35rem;flex-wrap:wrap;">
                                                                    {grp.clone().map(|g| view!{ <span class="badge">{g}</span> })}
                                                                    {if has_cosig {
                                                                        view!{
                                                                            <button
                                                                                type="button"
                                                                                on:click=move |_| set_show_cosig.update(|v| *v = !*v)
                                                                                style="background:rgba(59,130,246,.15);color:rgb(96,165,250);border:1px solid rgba(59,130,246,.3);padding:0.15rem 0.4rem;border-radius:4px;font-size:0.72rem;font-weight:600;cursor:pointer;"
                                                                            >
                                                                                {format!("+ {} cosig.", cosig_count)}
                                                                            </button>
                                                                        }.into_view()
                                                                    } else {
                                                                        view!{}.into_view()
                                                                    }}
                                                                </div>
                                                                {move || if show_cosig.get() && has_cosig {
                                                                    view!{
                                                                        <div style="margin-top:0.5rem;padding:0.5rem;background:rgba(255,255,255,.03);border-radius:4px;border-left:2px solid rgba(59,130,246,.5);">
                                                                            <div style="font-size:0.78rem;font-weight:700;color:var(--text-muted);margin-bottom:0.35rem;text-transform:uppercase;letter-spacing:0.05em;">"Cosignataires:"</div>
                                                                            {cosig_list.iter().map(|cos_id| {
                                                                                let cos_dep = dep_map_clone.get(cos_id);
                                                                                let (cos_name, cos_grp, cos_href_id) = deputy_display(cos_dep);
                                                                                let cos_dot = groupe_color(cos_grp.as_deref());
                                                                                let cos_href = cos_href_id.map(|x| app_href(&format!("/depute/{x}")));
                                                                                view!{
                                                                                    <div style="display:flex;align-items:center;gap:0.35rem;margin:0.25rem 0;font-size:0.82rem;">
                                                                                        <span class="amd-dot" style=format!("background:{};width:6px;height:6px;", cos_dot)></span>
                                                                                        {match cos_href {
                                                                                            Some(h) => view!{ <A href=h class="amd-link">{cos_name}</A> }.into_view(),
                                                                                            None => view!{ <span>{cos_name}</span> }.into_view(),
                                                                                        }}
                                                                                        {cos_grp.map(|g| view!{ <span class="badge" style="font-size:0.68rem;">{g}</span> })}
                                                                                    </div>
                                                                                }
                                                                            }).collect_view()}
                                                                        </div>
                                                                    }.into_view()
                                                                } else {
                                                                    view!{}.into_view()
                                                                }}
                                                            </div>
                                                        </div>
                                                    </td>
                                                    <td style="padding:.62rem .75rem;border-bottom:1px solid rgba(255,255,255,.06);font-size:.86rem;vertical-align:middle;">
                                                        <div style="display:flex;flex-direction:column;gap:0.12rem;">
                                                            <span style="font-weight:800;">{format!("Amd {}", amd_label)}</span>
                                                            <span style="color:var(--text-muted);font-size:.78rem;font-family:ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', monospace;">{e.id.clone()}</span>
                                                        </div>
                                                    </td>
                                                    <td style="padding:.62rem .75rem;border-bottom:1px solid rgba(255,255,255,.06);font-size:.86rem;vertical-align:middle;">
                                                        <div title=format!("{} — {}", dossier_id, dossier_title) style="display:flex;flex-direction:column;gap:0.12rem;min-width:0;">
                                                            <span class="amd-ellipsis">{dossier_title}</span>
                                                            <span style="color:var(--text-muted);font-size:.78rem;font-family:ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', monospace;">{dossier_id}</span>
                                                            {mission.as_ref().map(|m| view!{
                                                                <div style="margin-top:0.25rem;">
                                                                    <span class="badge" style="background:rgba(168,85,247,.12);border-color:rgba(168,85,247,.3);color:rgb(196,181,253);font-size:0.72rem;">
                                                                        {format!("Mission: {}", m)}
                                                                    </span>
                                                                </div>
                                                            })}
                                                        </div>
                                                    </td>
                                                    <td style="padding:.62rem .75rem;border-bottom:1px solid rgba(255,255,255,.06);font-size:.86rem;vertical-align:middle;">
                                                        <span class="badge">{e.art.clone().unwrap_or_else(|| "—".to_string())}</span>
                                                    </td>
                                                    <td style="padding:.62rem .75rem;border-bottom:1px solid rgba(255,255,255,.06);font-size:.86rem;vertical-align:middle;">
                                                        {sort_view}
                                                    </td>
                                                    <td style="padding:.62rem .75rem;border-bottom:1px solid rgba(255,255,255,.06);font-size:.84rem;vertical-align:middle;max-width:320px;">
                                                        {match expose {
                                                            Some(ref txt) if !txt.is_empty() => {
                                                                let display_txt = if txt.chars().count() > 200 {
                                                                    format!("{}…", txt.chars().take(200).collect::<String>())
                                                                } else {
                                                                    txt.clone()
                                                                };
                                                                view!{
                                                                    <div style="color:var(--text-secondary);line-height:1.35;word-break:break-word;" title=txt.clone()>
                                                                        {display_txt}
                                                                    </div>
                                                                }.into_view()
                                                            }
                                                            _ => view!{ <span style="color:var(--text-muted);">"—"</span> }.into_view(),
                                                        }}
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        }
                        .into_view()
                    }}

                    {move || {
                        if !show_undated.get() {
                            return view! {}.into_view();
                        }
                        // On n'affiche pas le contenu undated ici pour éviter un gros download.
                        // Le compteur + lien de fichier est déjà visible à gauche.
                        view! {
                            <div style="margin-top:0.9rem;color:var(--text-muted);font-size:0.82rem;line-height:1.35;">
                                <span class="badge">"Sans date"</span>
                                " activé. (Option : on pourra ajouter une section dédiée qui charge undated.json à la demande.)"
                            </div>
                        }.into_view()
                    }}
                </div>
            </div>

            <div style="margin-top:1rem;color:var(--text-muted);font-size:0.78rem;line-height:1.35;">
                {move || {
                    match index_res.get() {
                        Some(Ok(idx)) => format!("Données générées le {} • schema v{}", idx.generated_at, idx.schema_version),
                        _ => "".to_string(),
                    }
                }}
            </div>
        </div>
    }
}
