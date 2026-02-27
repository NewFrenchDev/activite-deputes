use leptos::*;
use leptos_router::A;
use std::collections::HashSet;

use crate::store::use_store;
use crate::models::*;
use crate::utils::{fmt_pct, matches_search, groupe_color, app_href};
use crate::components::{
    skeleton::SkeletonTable,
    period_selector::PeriodSelector,
    rate_bar::RateBar,
    tooltip::InfoIcon,
};

const PAGE_SIZE: usize = 50;

fn period_window_suffix(period: Period) -> &'static str {
    match period {
        Period::P30 => "sur les 30 derniers jours",
        Period::P180 => "sur les 180 derniers jours",
        Period::LEG => "sur la législature",
    }
}

fn period_window_short(period: Period) -> &'static str {
    match period {
        Period::P30 => "fenêtre 30 jours",
        Period::P180 => "fenêtre 180 jours",
        Period::LEG => "législature",
    }
}


#[component]
pub fn HomePage() -> impl IntoView {
    let store = use_store();

    let (period, set_period)               = create_signal(Period::P180);
    let (search, set_search)               = create_signal(String::new());
    let (filter_groupe, set_filter_groupe) = create_signal(String::new());
    let (sort_field, set_sort_field)       = create_signal(SortField::Participation);
    let (sort_dir, set_sort_dir)           = create_signal(SortDir::Desc);
    let (page, set_page)                   = create_signal(0usize);
    let (show_detail_cols, set_show_detail_cols) = create_signal(false);

    create_effect(move |_| {
        search.track();
        filter_groupe.track();
        period.track();
        sort_field.track();
        sort_dir.track();
        set_page.set(0);
    });

    let store_for_stats  = store.clone();
    let store_for_status = store.clone();
    let store_for_hero   = store.clone();
    let raw_stats = create_memo(move |_| {
        store_for_stats
            .stats_for(period.get())  // Retourne Resource
            .get()  //  Resource.get() → Option<Result<Vec>>
    });

    // Chiffres clés calculés depuis le dataset chargé (période sélectionnée)
    let hero_stats = create_memo(move |_| {
        let selected_period = period.get();
        let data = store_for_hero.stats_for(selected_period)
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default();
        if data.is_empty() {
            return None;
        }
        let nb = data.len();
        let mut rates: Vec<f64> = data.iter().map(|d| d.participation_rate).collect();
        rates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = if rates.len() % 2 == 1 {
            rates[rates.len() / 2]
        } else {
            let mid = rates.len() / 2;
            (rates[mid - 1] + rates[mid]) / 2.0
        };
        let total_amds: u32 = data.iter().map(|d| d.amd_authored).sum();
        let max_scrutins = data.iter().map(|d| d.scrutins_eligibles).max().unwrap_or(0);
        let nb_groupes = data.iter()
            .filter_map(|d| d.groupe_abrev.clone())
            .collect::<HashSet<_>>().len();
        Some((selected_period, nb, median, total_amds, max_scrutins, nb_groupes))
    });

    let all_groupes = create_memo(move |_| {
        let Some(Ok(ref data)) = raw_stats.get() else { return vec![]; };
        let mut set: Vec<String> = data.iter()
            .filter_map(|d| d.groupe_abrev.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        set.sort();
        set
    });

    let filtered_sorted = create_memo(move |_| {
        let Some(Ok(ref data)) = raw_stats.get() else { return vec![]; };
        let q   = search.get();
        let grp = filter_groupe.get();
        let sf  = sort_field.get();
        let sd  = sort_dir.get();

        let mut rows: Vec<DeputeStats> = data.iter()
            .filter(|d| {
                let full = format!("{} {} {} {}",
                    d.nom, d.prenom,
                    d.dept.as_deref().unwrap_or(""),
                    d.circo.as_deref().unwrap_or(""));
                matches_search(&full, &q)
                    && (grp.is_empty() || d.groupe_abrev.as_deref() == Some(grp.as_str()))
            })
            .cloned()
            .collect();

        rows.sort_by(|a, b| {
            let ord = match sf {
                SortField::Nom =>
                    a.nom.cmp(&b.nom).then(a.prenom.cmp(&b.prenom)),
                SortField::Groupe =>
                    a.groupe_abrev.cmp(&b.groupe_abrev),
                SortField::Participation =>
                    a.participation_rate.partial_cmp(&b.participation_rate)
                        .unwrap_or(std::cmp::Ordering::Equal),
                SortField::AmdsAuthored =>
                    a.amd_authored.cmp(&b.amd_authored),
                SortField::AmdsAdopted =>
                    a.amd_adopted.cmp(&b.amd_adopted),
                SortField::AmdAdoptionRate =>
                    a.amd_adoption_rate.unwrap_or(0.0)
                        .partial_cmp(&b.amd_adoption_rate.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal),
                SortField::ScrutinsEligibles =>
                    a.scrutins_eligibles.cmp(&b.scrutins_eligibles),
            };
            if sd == SortDir::Desc { ord.reverse() } else { ord }
        });
        rows
    });

    let top_bottom_participation = create_memo(move |_| {
        let mut rows: Vec<DeputeStats> = filtered_sorted
            .get()
            .into_iter()
            .filter(|d| d.scrutins_eligibles > 0)
            .collect();

        rows.sort_by(|a, b| {
            b.participation_rate
                .partial_cmp(&a.participation_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.scrutins_eligibles.cmp(&a.scrutins_eligibles))
                .then_with(|| a.nom.cmp(&b.nom))
                .then_with(|| a.prenom.cmp(&b.prenom))
        });

        let top = rows.iter().take(5).cloned().collect::<Vec<_>>();
        let bottom = rows.iter().rev().take(5).cloned().collect::<Vec<_>>();
        (top, bottom)
    });

    let total_count = move || filtered_sorted.get().len();
    let total_pages = move || ((total_count() + PAGE_SIZE - 1) / PAGE_SIZE).max(1);

    create_effect(move |_| {
        let total = filtered_sorted.get().len();
        let tp = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
        let last = tp.saturating_sub(1);
        set_page.update(|p| {
            if *p > last {
                *p = last;
            }
        });
    });

    let page_data = create_memo(move |_| {
        let start = page.get() * PAGE_SIZE;
        filtered_sorted.get().into_iter().skip(start).take(PAGE_SIZE).collect::<Vec<_>>()
    });

    let handle_sort = move |field: SortField| {
        if sort_field.get() == field {
            set_sort_dir.update(|d| *d = d.toggle());
        } else {
            set_sort_field.set(field);
            set_sort_dir.set(SortDir::Desc);
        }
    };

    // Nombre de colonnes selon l'état du toggle
    let col_count = move || if show_detail_cols.get() { 12 } else { 8 };

    view! {
        <div class="reveal">

            // ── Hero : chiffres clés ─────────────────────────────────────────
            {move || hero_stats.get().map(|(hero_period, nb, median, total_amds, max_scrutins, nb_groupes)| view! {
                <div style="margin-bottom:2rem;padding:1.5rem;background:linear-gradient(135deg,var(--bg-secondary) 0%,rgba(34,211,238,0.04) 100%);border:1px solid var(--bg-border);border-radius:12px;">
                    <div style="margin-bottom:1.25rem;">
                        <h1 style="font-size:1.5rem;font-weight:700;margin:0 0 0.3rem 0;line-height:1.2;">
                            "Activité parlementaire observable"
                        </h1>
                        <p style="color:var(--text-muted);font-size:0.82rem;margin:0;">
                            "Assemblée nationale · 17e législature · Données open data officielles"
                        </p>
                        <p style="color:var(--text-muted);font-size:0.74rem;margin:0.25rem 0 0 0;">
                            "Période affichée : "
                            <strong style="color:var(--text-secondary);">{move || period.get().label()}</strong>
                        </p>
                        {move || store_for_status.status.get().and_then(|r| r.ok()).map(|s| view! {
                            <p style="color:var(--text-muted);font-size:0.73rem;margin-top:0.2rem;">
                                "Mise à jour : "
                                <strong style="color:var(--text-secondary);">{s.last_update_readable}</strong>
                            </p>
                        })}
                    </div>

                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(140px,1fr));gap:0.75rem;">
                        <HeroStat
                            value=nb.to_string()
                            label="députés suivis"
                            sub="mandat actif 17e législature"
                        />
                        <HeroStat
                            value=format!("{:.1}%", median * 100.0)
                            label="participation médiane"
                            sub=period_window_suffix(hero_period)
                            accent=true
                        />
                        <HeroStat
                            value=fmt_thousands(total_amds)
                            label="amendements déposés"
                            sub=period_window_suffix(hero_period)
                        />
                        <HeroStat
                            value=max_scrutins.to_string()
                            label="scrutins publics"
                            sub=period_window_short(hero_period)
                        />
                        <HeroStat
                            value=nb_groupes.to_string()
                            label="groupes parlementaires"
                            sub="représentés dans le dataset"
                        />
                    </div>
                </div>
            })}

            // Si les données ne sont pas encore chargées, afficher un placeholder hero
            {move || if hero_stats.get().is_none() {
                view! {
                    <div style="margin-bottom:2rem;padding:1.5rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:12px;">
                        <div style="height:1.5rem;width:320px;border-radius:4px;" class="skeleton"></div>
                        <div style="height:0.8rem;width:220px;border-radius:4px;margin-top:0.5rem;" class="skeleton"></div>
                        <div style="display:grid;grid-template-columns:repeat(5,1fr);gap:0.75rem;margin-top:1.25rem;">
                            {(0..5).map(|_| view! {
                                <div style="height:72px;border-radius:8px;" class="skeleton"></div>
                            }).collect_view()}
                        </div>
                    </div>
                }.into_view()
            } else {
                view! { <span></span> }.into_view()
            }}

            // ── Filtres ──────────────────────────────────────────────────────
            <div class="home-filters" style="display:flex;flex-wrap:wrap;gap:0.75rem;align-items:center;margin-bottom:1rem;padding:0.85rem 1rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:8px;">
                <PeriodSelector period=period set_period=set_period />
                <div style="flex:1;min-width:200px;max-width:320px;">
                    <input
                        type="text"
                        placeholder="Rechercher nom, département…"
                        prop:value=move || search.get()
                        on:input=move |e| set_search.set(event_target_value(&e))
                        aria-label="Rechercher un député"
                    />
                </div>
                <select
                    prop:value=move || filter_groupe.get()
                    on:change=move |e| set_filter_groupe.set(event_target_value(&e))
                    style="width:auto;min-width:140px;"
                    aria-label="Filtrer par groupe"
                >
                    <option value="">"Tous les groupes"</option>
                    {move || all_groupes.get().into_iter().map(|g| {
                        let g2 = g.clone();
                        view! { <option value=g2>{g}</option> }
                    }).collect_view()}
                </select>
                <span style="font-size:0.75rem;color:var(--text-muted);margin-left:auto;">
                    {move || format!("{} député{}", total_count(), if total_count() > 1 { "s" } else { "" })}
                </span>
            </div>

            // ── Bandeau méthode ──────────────────────────────────────────────
            <div style="margin-bottom:0.75rem;padding:0.55rem 0.85rem;background:var(--accent-dim);border:1px solid var(--accent-border);border-radius:6px;font-size:0.75rem;color:var(--text-secondary);display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:0.5rem;">
                <span>
                    <strong style="color:var(--accent);">"ℹ Participation"</strong>
                    " : taux de positions enregistrées (Pour/Contre/Abstention) sur scrutins publics — pas une mesure de présence physique."
                    <A href=app_href("/methodologie") attr:style="color:var(--accent);margin-left:0.5rem;">"→ Méthode"</A>
                </span>
                // Toggle colonnes secondaires
                <button
                    on:click=move |_| set_show_detail_cols.update(|v| *v = !*v)
                    style="background:none;border:1px solid var(--bg-border);border-radius:5px;padding:0.25rem 0.6rem;cursor:pointer;font-size:0.73rem;color:var(--text-secondary);white-space:nowrap;"
                    title="Afficher ou masquer les colonnes Pour/Contre/Abst et Dept/Circo"
                >
                    {move || if show_detail_cols.get() { "− Moins de colonnes" } else { "+ Détail votes" }}
                </button>
            </div>

            // ── Top / bottom participation (sur la sélection courante) ─────
            {move || {
                let (top, bottom) = top_bottom_participation.get();
                if top.is_empty() && bottom.is_empty() {
                    return view! { <span></span> }.into_view();
                }

                let period_label = period.get().label().to_string();

                view! {
                    <div class="home-top5-grid" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:0.75rem;margin-bottom:0.9rem;">
                        <div style="background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:8px;padding:0.85rem;">
                            <div style="display:flex;align-items:center;justify-content:space-between;gap:0.5rem;margin-bottom:0.55rem;">
                                <strong style="font-size:0.85rem;">"Top 5 participation"</strong>
                                <span style="font-size:0.7rem;color:var(--text-muted);">{period_label.clone()}</span>
                            </div>
                            <div style="display:flex;flex-direction:column;gap:0.45rem;">
                                {top.into_iter().map(|d| {
                                    let id = d.deputy_id.clone();
                                    let nom = format!("{} {}", d.prenom, d.nom);
                                    let grp = d.groupe_abrev.clone().unwrap_or_else(|| "—".to_string());
                                    view! {
                                        <div style="display:grid;grid-template-columns:minmax(0,1fr) auto;gap:0.5rem;align-items:center;">
                                            <div style="min-width:0;">
                                                <A href=app_href(&format!("/depute/{id}")) attr:style="color:var(--text-primary);text-decoration:none;font-size:0.8rem;font-weight:500;display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">
                                                    {nom}
                                                </A>
                                                <span style="font-size:0.68rem;color:var(--text-muted);">{grp}</span>
                                            </div>
                                            <div><RateBar rate=d.participation_rate /></div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>

                        <div style="background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:8px;padding:0.85rem;">
                            <div style="display:flex;align-items:center;justify-content:space-between;gap:0.5rem;margin-bottom:0.55rem;">
                                <strong style="font-size:0.85rem;">"5 participations les plus faibles"</strong>
                                <span style="font-size:0.7rem;color:var(--text-muted);">{period_label}</span>
                            </div>
                            <div style="display:flex;flex-direction:column;gap:0.45rem;">
                                {bottom.into_iter().map(|d| {
                                    let id = d.deputy_id.clone();
                                    let nom = format!("{} {}", d.prenom, d.nom);
                                    let grp = d.groupe_abrev.clone().unwrap_or_else(|| "—".to_string());
                                    view! {
                                        <div style="display:grid;grid-template-columns:minmax(0,1fr) auto;gap:0.5rem;align-items:center;">
                                            <div style="min-width:0;">
                                                <A href=app_href(&format!("/depute/{id}")) attr:style="color:var(--text-primary);text-decoration:none;font-size:0.8rem;font-weight:500;display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">
                                                    {nom}
                                                </A>
                                                <span style="font-size:0.68rem;color:var(--text-muted);">{grp}</span>
                                            </div>
                                            <div><RateBar rate=d.participation_rate /></div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    </div>
                }.into_view()
            }}

            // ── Tableau ──────────────────────────────────────────────────────
            <div class="deputies-table-wrap" style="overflow-x:auto;background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:8px;">
                <table class="data-table deputies-table" role="grid" aria-label="Tableau des députés et leur activité parlementaire">
                    <thead>
                        <tr>
                            <SortTh label="Député"      field=SortField::Nom               sf=sort_field sd=sort_dir handle_sort=handle_sort />
                            <SortTh label="Groupe"      field=SortField::Groupe            sf=sort_field sd=sort_dir handle_sort=handle_sort />
                            <th>
                                <span style="display:inline-flex;align-items:center;gap:0.25rem;">
                                    "Participation"
                                    <InfoIcon text="Taux de votes exprimés (Pour/Contre/Abstention) sur scrutins publics où le député avait un mandat actif. Ne mesure pas la présence physique." />
                                </span>
                            </th>
                            <SortTh label="Scrutins"    field=SortField::ScrutinsEligibles sf=sort_field sd=sort_dir handle_sort=handle_sort />
                            // Colonnes détail — masquées par défaut
                            {move || if show_detail_cols.get() { view! {
                                <>
                                    <th>"Pour"</th>
                                    <th>"Contre"</th>
                                    <th>"Abst."</th>
                                </>
                            }.into_view() } else { view! { <></> }.into_view() }}
                            <SortTh label="Amendements" field=SortField::AmdsAuthored      sf=sort_field sd=sort_dir handle_sort=handle_sort />
                            <SortTh label="Adoptés"     field=SortField::AmdsAdopted       sf=sort_field sd=sort_dir handle_sort=handle_sort />
                            <th>
                                <span style="display:inline-flex;align-items:center;gap:0.25rem;">
                                    "Taux adoption"
                                    <InfoIcon text="Amendements adoptés / amendements déposés en auteur principal. Vide si 0 amendements." />
                                </span>
                            </th>
                            {move || if show_detail_cols.get() { view! {
                                <th>"Dept / Circo"</th>
                            }.into_view() } else { view! { <></> }.into_view() }}
                            <th>"Fiche"</th>
                        </tr>
                    </thead>
                    {move || match raw_stats.get() {
                        None => view! { <SkeletonTable rows=20 cols=col_count() /> }.into_view(),
                        Some(Err(ref e)) => view! {
                            <tbody><tr><td colspan=col_count().to_string() style="text-align:center;padding:2rem;color:var(--danger);">
                                {format!("Erreur de chargement : {e}")}
                            </td></tr></tbody>
                        }.into_view(),
                        Some(Ok(_)) => view! {
                            <tbody>
                                {move || {
                                    let rows = page_data.get();
                                    if total_count() == 0 {
                                        return view! {
                                            <tr>
                                                <td colspan=col_count().to_string() style="text-align:center;padding:1.4rem 1rem;color:var(--text-secondary);">
                                                    <div style="display:flex;flex-direction:column;gap:0.45rem;align-items:center;">
                                                        <strong>"Aucun député ne correspond à la recherche ou aux filtres."</strong>
                                                        <button class="btn"
                                                            on:click=move |_| {
                                                                set_search.set(String::new());
                                                                set_filter_groupe.set(String::new());
                                                                set_page.set(0);
                                                            }
                                                            style="font-size:0.75rem;padding:0.3rem 0.7rem;">
                                                            "Réinitialiser les filtres"
                                                        </button>
                                                    </div>
                                                </td>
                                            </tr>
                                        }.into_view();
                                    }

                                    rows.into_iter().map(|d| {
                                    let id         = d.deputy_id.clone();
                                    let adopt_rate = d.amd_adoption_rate;
                                    let grp_color  = groupe_color(d.groupe_abrev.as_deref());
                                    let grp_title  = d.groupe_nom.clone().unwrap_or_default();
                                    let dept_label = d.dept.clone().unwrap_or_default();
                                    let show_det   = show_detail_cols.get();
                                    let nom_display = format!("{} {}", d.prenom, d.nom);
                                    let nom_aria    = format!("Voir la fiche de {} {}", d.prenom, d.nom);
                                    view! {
                                        <tr>
                                            <td class="td-nom">
                                                <A href=app_href(&format!("/depute/{id}"))>
                                                    {nom_display}
                                                </A>
                                            </td>
                                            <td class="td-groupe">
                                                {d.groupe_abrev.as_ref().map(|g| view! {
                                                    <span class="badge"
                                                        style=format!("border-color:{grp_color};color:{grp_color};")
                                                        title=grp_title.clone()>
                                                        {g.clone()}
                                                    </span>
                                                })}
                                            </td>
                                            <td class="td-participation"><RateBar rate=d.participation_rate /></td>
                                            <td style="font-variant-numeric:tabular-nums;color:var(--text-secondary);font-size:0.8rem;">
                                                {d.votes_exprimes}"/"{d.scrutins_eligibles}
                                            </td>
                                            {if show_det { view! {
                                                <>
                                                    <td style="font-size:0.8rem;color:var(--success);">{d.pour_count}</td>
                                                    <td style="font-size:0.8rem;color:var(--danger);">{d.contre_count}</td>
                                                    <td style="font-size:0.8rem;color:var(--warning);">{d.abst_count}</td>
                                                </>
                                            }.into_view() } else { view! { <></> }.into_view() }}
                                            <td class="td-amd-authored" style="font-variant-numeric:tabular-nums;">{d.amd_authored}</td>
                                            <td class="td-amd-adopted" style="font-variant-numeric:tabular-nums;color:var(--success);">{d.amd_adopted}</td>
                                            <td class="td-amd-rate" style="font-size:0.8rem;color:var(--text-secondary);">
                                                {adopt_rate.map(|r| fmt_pct(r)).unwrap_or_else(|| "—".to_string())}
                                            </td>
                                            {if show_det { view! {
                                                <td style="font-size:0.78rem;color:var(--text-muted);">
                                                    {dept_label}
                                                    {d.circo.as_ref().map(|c| format!(" #{c}"))}
                                                </td>
                                            }.into_view() } else { view! { <></> }.into_view() }}
                                            <td class="td-fiche">
                                                <A href=app_href(&format!("/depute/{id}")) class="btn"
                                                    attr:style="padding:0.25rem 0.6rem;font-size:0.75rem;"
                                                    attr:aria-label=nom_aria>
                                                    "→"
                                                </A>
                                            </td>
                                        </tr>
                                    }
                                    }).collect_view()
                                }}
                            </tbody>
                        }.into_view(),
                    }}
                </table>
            </div>

            // ── Pagination ───────────────────────────────────────────────────
            {move || {
                let tp = total_pages();
                let _cp = page.get();
                if tp <= 1 { return view! { <span></span> }.into_view(); }
                view! {
                    <div role="navigation" aria-label="Pagination"
                        style="display:flex;align-items:center;justify-content:center;gap:0.5rem;margin-top:1rem;">
                        <button class="btn"
                            prop:disabled={move || page.get() == 0}
                            on:click=move |_| set_page.update(|p| *p = p.saturating_sub(1))
                        >
                            "← Précédent"
                        </button>
                        <span style="font-size:0.8rem;color:var(--text-muted);">
                            {move || format!("Page {} / {}", page.get() + 1, total_pages())}
                        </span>
                        <button class="btn"
                            prop:disabled={move || page.get() + 1 >= total_pages()}
                            on:click=move |_| {
                                let last = total_pages().saturating_sub(1);
                                set_page.update(|p| *p = (*p + 1).min(last));
                            }
                        >
                            "Suivant →"
                        </button>
                    </div>
                }.into_view()
            }}

            <p style="font-size:0.72rem;color:var(--text-muted);margin-top:0.75rem;text-align:right;">
                "Cliquez sur les en-têtes pour trier · "
                <A href=app_href("/exporter") attr:style="color:var(--accent);">"↓ Télécharger CSV"</A>
            </p>
        </div>
    }
}

// ── Composants hero ──────────────────────────────────────────────────────────

#[component]
fn HeroStat(
    value: String,
    label: &'static str,
    sub: &'static str,
    #[prop(optional)] accent: bool,
) -> impl IntoView {
    view! {
        <div style="padding:0.85rem 1rem;background:var(--bg-primary);border:1px solid var(--bg-border);border-radius:10px;">
            <div style=format!(
                "font-size:1.5rem;font-weight:700;line-height:1.1;margin-bottom:0.2rem;color:{};",
                if accent { "var(--accent)" } else { "var(--text-primary)" }
            )>
                {value}
            </div>
            <div style="font-size:0.75rem;font-weight:600;color:var(--text-secondary);margin-bottom:0.1rem;">{label}</div>
            <div style="font-size:0.68rem;color:var(--text-muted);line-height:1.3;">{sub}</div>
        </div>
    }
}

fn fmt_thousands(n: u32) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { result.push('\u{202f}'); } // espace fine insécable
        result.push(c);
    }
    result.chars().rev().collect()
}

// ── SortTh ───────────────────────────────────────────────────────────────────

#[component]
fn SortTh<F>(
    label: &'static str,
    field: SortField,
    sf: ReadSignal<SortField>,
    sd: ReadSignal<SortDir>,
    handle_sort: F,
) -> impl IntoView
where
    F: Fn(SortField) + 'static + Clone,
{
    view! {
        <th
            on:click=move |_| handle_sort(field)
            class=move || if sf.get() == field { "sorted" } else { "" }
            style="cursor:pointer;"
            aria-sort=move || match (sf.get() == field, sd.get()) {
                (true, SortDir::Asc)  => "ascending",
                (true, SortDir::Desc) => "descending",
                _                     => "none",
            }
        >
            {label}
            {move || if sf.get() == field {
                if sd.get() == SortDir::Desc { " ↓" } else { " ↑" }
            } else { "" }}
        </th>
    }
}
