use leptos::*;
use leptos_router::*;

use crate::api::{base_url, fetch_deputy_ppl_shard};
use crate::store::use_store;
use crate::models::*;
use crate::utils::{fmt_pct, groupe_color, participation_class, app_href};
use crate::components::{
    kpi_card::KpiCard,
    period_selector::PeriodSelector,
    skeleton::SkeletonKpi,
    tooltip::InfoIcon,
};

fn period_query_value(period: Period) -> &'static str {
    match period {
        Period::P30 => "p30",
        Period::P180 => "p180",
        Period::Leg => "leg",
    }
}

#[component]
pub fn DeputePage() -> impl IntoView {
    let store  = use_store();
    let params = use_params_map();
    let params_for_ppl = params.clone();
    let dep_id = move || params.with(|p| p.get("id").cloned().unwrap_or_default());

    let (period, set_period) = create_signal(Period::P180);
    let (show_photo, set_show_photo) = create_signal(true);

    let deputy_ppl_res = create_resource(
        move || params_for_ppl.with(|p| p.get("id").cloned().unwrap_or_default()),
        |id| async move {
            if id.trim().is_empty() {
                Ok(None)
            } else {
                fetch_deputy_ppl_shard(&id).await
            }
        },
    );

    create_effect(move |_| {
        let _ = period.get();
        let _ = dep_id();
        set_show_photo.set(true);
    });

    // Chercher le député dans le store (pas de re-fetch)
    let store_for_depute = store.clone();
    let depute = create_memo(move |_| {
        let id = dep_id();
        store_for_depute.find_depute(period.get(), &id)
    });

    view! {
        <div>
            <div style="margin-bottom:1rem;">
                <A href=app_href("/") attr:style="color:var(--accent);font-size:0.82rem;text-decoration:none;">
                    "← Retour au tableau"
                </A>
            </div>

            {move || {
                let resource = store.stats_for(period.get());
                match resource.get() {
                    None => view! {
                        <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(180px,1fr));gap:1rem;margin-bottom:1.5rem;">
                            <SkeletonKpi/><SkeletonKpi/><SkeletonKpi/><SkeletonKpi/>
                        </div>
                    }.into_view(),
                    Some(Err(ref e)) => view! {
                        <p style="color:var(--danger);">{format!("Erreur : {e}")}</p>
                    }.into_view(),
                    Some(Ok(_)) => match depute.get() {
                        None => view! {
                            <div style="text-align:center;padding:3rem;color:var(--text-muted);">
                                <p style="font-size:1.2rem;margin-bottom:0.5rem;">"Député non trouvé"</p>
                                <p style="font-size:0.82rem;">"L'identifiant "{dep_id()}" est introuvable dans les données de la période sélectionnée."</p>
                                <A href=app_href("/") class="btn" attr:style="margin-top:1rem;">"Retour à l'accueil"</A>
                            </div>
                        }.into_view(),
                        Some(d) => {
                            let grp_color   = groupe_color(d.groupe_abrev.as_deref());
                            let part_class  = participation_class(d.participation_rate);
                            let current_period = period.get();
                            let current_stats = store
                                .stats_for(current_period)
                                .get()
                                .and_then(|r| r.ok())
                                .unwrap_or_default();
                            let benchmarks = compute_benchmarks(&d, &current_stats);
                            let period_snapshots = collect_period_snapshots(&store, &d.deputy_id);
                            let dataset_json_url = format!("{}/{}", base_url(), current_period.json_file());
                            let dataset_csv_url = format!("{}/{}", base_url(), current_period.csv_file());
                            let d = d.clone();
                            let deputy_ppl_res = deputy_ppl_res.clone();
                            let photo_url = an_photo_url(&d.deputy_id).unwrap_or_default();
                            let has_photo = !photo_url.is_empty();
                            let profile_url = an_profile_url(&d.deputy_id);
                            let initials = initials(&d.prenom, &d.nom);
                            let photo_alt = format!("Photo de {} {}", d.prenom, d.nom);
                            let location_line = dept_circo_label(d.dept.as_deref(), d.circo.as_deref());
                            let naissance_line = match (d.date_naissance, d.pays_naissance.clone()) {
                                (Some(date), Some(pays)) if !pays.trim().is_empty() => Some(format!("{} · {}", date, pays.trim())),
                                (Some(date), _) => Some(date.to_string()),
                                (None, Some(pays)) if !pays.trim().is_empty() => Some(pays.trim().to_string()),
                                _ => None,
                            };
                            let website_href = d.site_web.as_deref().map(normalize_external_url);
                            let extra_site_labels: Vec<String> = if !d.sites_web_sources.is_empty() {
                                let mut labels: Vec<String> = Vec::new();
                                for entry in &d.sites_web_sources {
                                    let is_main = entry
                                        .url
                                        .as_deref()
                                        .map(normalize_external_url)
                                        .filter(|u| !u.is_empty())
                                        .as_ref()
                                        .map(|u| match website_href.as_ref() {
                                            Some(main) => main == u,
                                            None => false,
                                        })
                                        .unwrap_or(false);
                                    if is_main {
                                        continue;
                                    }
                                    let label = extra_site_label_from_source(entry);
                                    if !label.is_empty() && !labels.iter().any(|x| x == &label) {
                                        labels.push(label);
                                    }
                                }
                                labels
                            } else {
                                d.sites_web
                                    .iter()
                                    .filter_map(|u| {
                                        let normalized = normalize_external_url(u);
                                        match website_href.as_ref() {
                                            Some(main) if main == &normalized => None,
                                            _ => Some(extra_site_label(&normalized)),
                                        }
                                    })
                                    .fold(Vec::new(), |mut acc, label| {
                                        if !label.is_empty() && !acc.iter().any(|x| x == &label) {
                                            acc.push(label);
                                        }
                                        acc
                                    })
                            };
                            let hatvp_href = d.uri_hatvp.as_deref().map(normalize_external_url);

                            view! {
                                <div class="reveal">
                                    // Header enrichi (photo + liens + identité)
                                    <div style=format!("margin-bottom:1.5rem;padding-bottom:1.5rem;padding-left:0.9rem;border-bottom:1px solid var(--bg-border);border-left:4px solid {};", grp_color)>
                                        <div style="display:flex;align-items:flex-start;justify-content:space-between;flex-wrap:wrap;gap:1rem;">
                                            <div style="display:flex;align-items:flex-start;gap:1rem;flex-wrap:wrap;">
                                                <div style=format!("width:96px;height:96px;border-radius:14px;overflow:hidden;border:1px solid {}33;background:var(--bg-secondary);display:flex;align-items:center;justify-content:center;box-shadow:inset 0 0 0 1px rgba(255,255,255,.02), 0 0 0 1px {}22;", grp_color, grp_color)>
                                                    {move || {
                                                        if show_photo.get() && has_photo {
                                                            view! {
                                                                <img
                                                                    src=photo_url.clone()
                                                                    alt=photo_alt.clone()
                                                                    style="width:100%;height:100%;object-fit:cover;display:block;"
                                                                    on:error=move |_| set_show_photo.set(false)
                                                                />
                                                            }.into_view()
                                                        } else {
                                                            view! {
                                                                <span style="font-size:1.35rem;font-weight:700;color:var(--accent);letter-spacing:0.03em;">
                                                                    {initials.clone()}
                                                                </span>
                                                            }.into_view()
                                                        }
                                                    }}
                                                </div>

                                                <div>
                                                    <h1 style="font-size:1.6rem;font-weight:700;margin:0 0 0.4rem 0;">
                                                        {format!("{} {}", d.prenom, d.nom)}
                                                    </h1>
                                                    <div style="display:flex;align-items:center;gap:0.75rem;flex-wrap:wrap;margin-bottom:0.45rem;">
                                                        {d.groupe_abrev.as_ref().map(|g| view! {
                                                            <span class="badge"
                                                                style=format!("border-color:{grp_color};color:{grp_color};font-size:0.75rem;padding:0.15rem 0.5rem;")>
                                                                {g.clone()}
                                                            </span>
                                                        })}
                                                        {d.groupe_nom.as_ref().map(|g| view! {
                                                            <span style="color:var(--text-secondary);font-size:0.82rem;">{g.clone()}</span>
                                                        })}
                                                        <span style="font-size:0.74rem;color:var(--text-muted);padding:0.15rem 0.45rem;border:1px solid var(--bg-border);border-radius:999px;">
                                                            {d.deputy_id.clone()}
                                                        </span>
                                                    </div>

                                                    <div style="display:flex;align-items:center;gap:0.4rem;flex-wrap:wrap;margin-bottom:0.25rem;">
                                                        <span style="color:var(--text-muted);font-size:0.78rem;">"Parti :"</span>
                                                        <span style="color:var(--text-secondary);font-size:0.78rem;">
                                                            {d.parti_rattachement.clone().unwrap_or_else(|| "Non disponible".to_string())}
                                                        </span>
                                                        <InfoIcon text="Le groupe parlementaire est le regroupement officiel à l'Assemblée. Le parti de rattachement est l'organisation politique déclarée — peut être absent ou différent du groupe." />
                                                    </div>

                                                    {location_line.clone().map(|line| view! {
                                                        <p style="color:var(--text-muted);font-size:0.78rem;margin:0 0 0.3rem 0;">{line}</p>
                                                    })}

                                                    <div style="display:flex;gap:0.5rem;flex-wrap:wrap;margin-top:0.6rem;">
                                                        <a
                                                            href=profile_url.clone()
                                                            target="_blank"
                                                            rel="noopener noreferrer"
                                                            style="font-size:0.75rem;padding:0.35rem 0.65rem;border:1px solid var(--bg-border);border-radius:8px;background:var(--bg-secondary);color:var(--text-primary);text-decoration:none;">
                                                            "Profil AN ↗"
                                                        </a>
                                                        <A
                                                            href={
                                                                let compare_deputy_id = d.deputy_id.clone();
                                                                move || format!(
                                                                    "/comparer?a={}&period={}",
                                                                    compare_deputy_id, period_query_value(period.get()))
                                                            }
                                                            attr:style="font-size:0.75rem;padding:0.35rem 0.65rem;border:1px solid var(--bg-border);border-radius:8px;background:var(--bg-secondary);color:var(--text-primary);text-decoration:none;">
                                                            "Comparer ↔"
                                                        </A>
                                                        {d.email_assemblee.as_ref().map(|mail| view! {
                                                            <a
                                                                href=format!("mailto:{}", mail)
                                                                style="font-size:0.75rem;padding:0.35rem 0.65rem;border:1px solid var(--bg-border);border-radius:8px;background:var(--bg-secondary);color:var(--text-primary);text-decoration:none;">
                                                                "Email AN"
                                                            </a>
                                                        })}
                                                        {website_href.clone().map(|url| view! {
                                                            <a
                                                                href=url
                                                                target="_blank"
                                                                rel="noopener noreferrer"
                                                                style="font-size:0.75rem;padding:0.35rem 0.65rem;border:1px solid var(--bg-border);border-radius:8px;background:var(--bg-secondary);color:var(--text-primary);text-decoration:none;">
                                                                "Site web ↗"
                                                            </a>
                                                        })}
                                                        {d.telephones.first().cloned().map(|tel| view! {
                                                            <span
                                                                style="font-size:0.75rem;padding:0.35rem 0.65rem;border:1px solid var(--bg-border);border-radius:8px;background:var(--bg-secondary);color:var(--text-primary);text-decoration:none;">
                                                                {format!("Tél. {}", tel)}
                                                            </span>
                                                        })}
                                                        {hatvp_href.clone().map(|url| view! {
                                                            <a
                                                                href=url
                                                                target="_blank"
                                                                rel="noopener noreferrer"
                                                                style="font-size:0.75rem;padding:0.35rem 0.65rem;border:1px solid var(--bg-border);border-radius:8px;background:var(--bg-secondary);color:var(--text-primary);text-decoration:none;">
                                                                "HATVP ↗"
                                                            </a>
                                                        })}
                                                    </div>
                                                </div>
                                            </div>

                                            <div style="display:flex;flex-direction:column;gap:0.75rem;align-items:flex-end;min-width:260px;">
                                                <PeriodSelector period=period set_period=set_period />
                                                <div style=format!("padding:0.75rem 0.9rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-left:3px solid {};border-radius:10px;min-width:260px;", grp_color)>
                                                    <div style="font-size:0.72rem;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;margin-bottom:0.35rem;">"Période calculée"</div>
                                                    <div style="font-size:0.82rem;color:var(--text-secondary);line-height:1.4;">
                                                        <strong style="color:var(--text-primary);">{format!("{} → {}", d.period_start, d.period_end)}</strong>
                                                        <div style="margin-top:0.2rem;">{format!("{} scrutins éligibles", d.scrutins_eligibles)}</div>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    // KPIs
                                    <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(175px,1fr));gap:1rem;margin-bottom:1.75rem;">
                                        <KpiCard
                                            label="Participation scrutins"
                                            value=fmt_pct(d.participation_rate)
                                            sub=format!("{} / {} scrutins éligibles", d.votes_exprimes, d.scrutins_eligibles)
                                            color=part_class
                                        />
                                        <KpiCard
                                            label="Votes POUR"
                                            value=d.pour_count.to_string()
                                            sub="positions enregistrées".to_string()
                                            color="var(--success)"
                                        />
                                        <KpiCard
                                            label="Votes CONTRE"
                                            value=d.contre_count.to_string()
                                            sub="positions enregistrées".to_string()
                                            color="var(--danger)"
                                        />
                                        <KpiCard
                                            label="Abstentions"
                                            value=d.abst_count.to_string()
                                            sub="positions enregistrées".to_string()
                                            color="var(--warning)"
                                        />
                                        <KpiCard
                                            label="Non-votant"
                                            value=d.non_votant.to_string()
                                            sub="délégation / absence déclarée".to_string()
                                        />
                                        <KpiCard
                                            label="Absences"
                                            value=d.absent.to_string()
                                            sub="aucune position enregistrée".to_string()
                                        />
                                        <KpiCard
                                            label="Amendements déposés"
                                            value=d.amd_authored.to_string()
                                            sub=format!("{} cosignés", d.amd_cosigned)
                                        />
                                        <KpiCard
                                            label="Amendements adoptés"
                                            value=d.amd_adopted.to_string()
                                            sub=d.amd_adoption_rate
                                                .map(|r| format!("taux : {}", fmt_pct(r)))
                                                .unwrap_or_else(|| "—".to_string())
                                            color="var(--success)"
                                        />
                                    </div>

                                    // Lecture relative (benchmark dataset + groupe)
                                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:1rem;margin-bottom:1.75rem;">
                                        <div style=format!("padding:1rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-left:3px solid {};border-radius:10px;", grp_color)>
                                            <h2 style="font-size:0.82rem;font-weight:600;margin:0 0 .75rem 0;text-transform:uppercase;letter-spacing:.06em;color:var(--text-muted);">"Repères de comparaison"</h2>
                                            <div style="display:flex;flex-direction:column;gap:.55rem;font-size:.8rem;">
                                                <BenchmarkRow
                                                    label="Participation"
                                                    value=d.participation_rate
                                                    median=benchmarks.participation_median
                                                    group_avg=benchmarks.participation_group_avg
                                                    is_percent=true
                                                />
                                                <BenchmarkRow
                                                    label="Amendements déposés"
                                                    value={d.amd_authored as f64}
                                                    median=benchmarks.amd_authored_median
                                                    group_avg=benchmarks.amd_authored_group_avg
                                                    is_percent=false
                                                />
                                                <BenchmarkRow
                                                    label="Amendements adoptés"
                                                    value={d.amd_adopted as f64}
                                                    median=benchmarks.amd_adopted_median
                                                    group_avg=benchmarks.amd_adopted_group_avg
                                                    is_percent=false
                                                />
                                                <BenchmarkRow
                                                    label="Taux d'adoption"
                                                    value=d.amd_adoption_rate.unwrap_or(0.0)
                                                    median=benchmarks.amd_adoption_rate_median
                                                    group_avg=benchmarks.amd_adoption_rate_group_avg
                                                    is_percent=true
                                                />
                                            </div>
                                        </div>

                                        <div style=format!("padding:1rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-left:3px solid {};border-radius:10px;", grp_color)>
                                            <h2 style="font-size:0.82rem;font-weight:600;margin:0 0 .75rem 0;text-transform:uppercase;letter-spacing:.06em;color:var(--text-muted);">"Tendance multi-périodes"</h2>
                                            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:.75rem;">
                                                {period_snapshots.iter().cloned().map(|snap| view! {
                                                    <div style="padding:.75rem;border:1px solid var(--bg-border);border-radius:8px;background:rgba(255,255,255,0.01);">
                                                        <div style="font-size:.72rem;text-transform:uppercase;letter-spacing:.05em;color:var(--text-muted);margin-bottom:.35rem;">
                                                            {snap.period.label().to_string()}
                                                        </div>
                                                        <div style="font-size:.95rem;font-weight:600;color:var(--text-primary);margin-bottom:.15rem;">
                                                            {fmt_pct(snap.participation_rate)}
                                                        </div>
                                                        <div style="font-size:.75rem;color:var(--text-muted);line-height:1.45;">
                                                            {format!("Participation · {} scrutins", snap.scrutins_eligibles)}
                                                        </div>
                                                        <div style="font-size:.75rem;color:var(--text-muted);line-height:1.45;margin-top:.35rem;">
                                                            {format!("Amd: {} déposés / {} adoptés", snap.amd_authored, snap.amd_adopted)}
                                                        </div>
                                                    </div>
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    </div>

                                    // Cartes identité / contact
                                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:1rem;margin-bottom:1.75rem;">
                                        <div style=format!("padding:1rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-left:3px solid {};border-radius:10px;", grp_color)>
                                            <h2 style="font-size:0.82rem;font-weight:600;margin:0 0 .75rem 0;text-transform:uppercase;letter-spacing:.06em;color:var(--text-muted);">"Identité & mandat"</h2>
                                            <div style="display:grid;grid-template-columns:auto 1fr;gap:.45rem .75rem;font-size:.82rem;align-items:start;">
                                                <span style="color:var(--text-muted);">"ID"</span><span style="font-family:monospace;">{d.deputy_id.clone()}</span>
                                                <span style="color:var(--text-muted);">"Groupe"</span><span>{d.groupe_nom.clone().unwrap_or_else(|| "—".into())}</span>
                                                <span style="color:var(--text-muted);">"Parti"</span><span>{d.parti_rattachement.clone().unwrap_or_else(|| "—".into())}</span>
                                                <span style="color:var(--text-muted);">"Profession"</span><span>{d.profession.clone().unwrap_or_else(|| "—".into())}</span>
                                                <span style="color:var(--text-muted);">"Naissance"</span><span>{naissance_line.clone().unwrap_or_else(|| "—".into())}</span>
                                                <span style="color:var(--text-muted);">"Territoire"</span><span>{location_line.clone().unwrap_or_else(|| "—".into())}</span>
                                                <span style="color:var(--text-muted);">"Mandat en cours (début)"</span><span>{d.mandat_debut.map(|x| x.to_string()).unwrap_or_else(|| "—".into())}</span>
                                                {if d.mandat_debut_legislature.is_some() && d.mandat_debut_legislature != d.mandat_debut {
                                                    view! {
                                                        <>
                                                            <span style="color:var(--text-muted);">"1er mandat AN (L17)"</span>
                                                            <span>{d.mandat_debut_legislature.map(|x| x.to_string()).unwrap_or_else(|| "—".into())}</span>
                                                        </>
                                                    }.into_view()
                                                } else {
                                                    ().into_view()
                                                }}
                                                {if d.mandat_assemblee_episodes.len() > 1 {
                                                    let summary = format_mandat_episodes_summary(&d.mandat_assemblee_episodes);
                                                    view! {
                                                        <>
                                                            <span style="color:var(--text-muted);">"Épisodes AN (L17)"</span>
                                                            <span style="line-height:1.35;">{summary}</span>
                                                        </>
                                                    }.into_view()
                                                } else {
                                                    ().into_view()
                                                }}
                                            </div>
                                        </div>

                                        <div style=format!("padding:1rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-left:3px solid {};border-radius:10px;", grp_color)>
                                            <h2 style="font-size:0.82rem;font-weight:600;margin:0 0 .75rem 0;text-transform:uppercase;letter-spacing:.06em;color:var(--text-muted);">"Contacts & sources"</h2>
                                            <div style="display:grid;grid-template-columns:auto 1fr;gap:.45rem .75rem;font-size:.82rem;align-items:start;">
                                                <span style="color:var(--text-muted);">"Assemblée"</span>
                                                <a href=profile_url.clone() target="_blank" rel="noopener noreferrer" style="color:var(--accent);text-decoration:none;">"Page député ↗"</a>

                                                <span style="color:var(--text-muted);">"Email"</span>
                                                {match d.email_assemblee.clone() {
                                                    Some(mail) => view! {
                                                        <a href=format!("mailto:{}", mail.clone()) style="color:var(--accent);text-decoration:none;word-break:break-word;">{mail}</a>
                                                    }.into_view(),
                                                    None => view! { <span style="color:var(--text-muted);">"Non publié dans ce dataset"</span> }.into_view(),
                                                }}

                                                <span style="color:var(--text-muted);">"Site web"</span>
                                                {match website_href.clone() {
                                                    Some(url) => view! {
                                                        <a href=url.clone() target="_blank" rel="noopener noreferrer" style="color:var(--accent);text-decoration:none;word-break:break-word;">{display_host(&url)}</a>
                                                    }.into_view(),
                                                    None => view! { <span style="color:var(--text-muted);">"—"</span> }.into_view(),
                                                }}

                                                <span style="color:var(--text-muted);">"Téléphone"</span>
                                                {match d.telephones.first().cloned() {
                                                    Some(tel) => view! {
                                                        <span style="color:var(--text-secondary);word-break:break-word;">{tel}</span>
                                                    }.into_view(),
                                                    None => view! { <span style="color:var(--text-muted);">"—"</span> }.into_view(),
                                                }}

                                                {if !extra_site_labels.is_empty() {
                                                    view! {
                                                        <>
                                                            <span style="color:var(--text-muted);">"Autres sites"</span>
                                                            <div style="display:flex;flex-direction:column;gap:.25rem;">
                                                                {extra_site_labels.iter().cloned().map(|label| {
                                                                    view! {
                                                                        <span style="color:var(--text-secondary);word-break:break-word;">{label}</span>
                                                                    }
                                                                }).collect_view()}
                                                            </div>
                                                        </>
                                                    }.into_view()
                                                } else {
                                                    view! { <></> }.into_view()
                                                }}

                                                <span style="color:var(--text-muted);">"HATVP"</span>
                                                {match hatvp_href.clone() {
                                                    Some(url) => view! {
                                                        <a href=url target="_blank" rel="noopener noreferrer" style="color:var(--accent);text-decoration:none;">"Lien déclaration ↗"</a>
                                                    }.into_view(),
                                                    None => view! { <span style="color:var(--text-muted);">"—"</span> }.into_view(),
                                                }}
                                            </div>
                                        </div>
                                    </div>

                                    // Propositions de loi associées (AN-only, shards backend V4.1)
                                    <div style=format!("margin-bottom:1.75rem;padding:1.0rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-left:3px solid {};border-radius:10px;", grp_color)>
                                        <div style="display:flex;align-items:flex-start;justify-content:space-between;gap:.75rem;flex-wrap:wrap;margin-bottom:.6rem;">
                                            <div>
                                                <h2 style="font-size:0.85rem;font-weight:600;margin:0 0 .2rem 0;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);">
                                                    "Propositions de loi associées (AN)"
                                                </h2>
                                                <p style="margin:0;color:var(--text-muted);font-size:.75rem;line-height:1.35;">
                                                    "Source: shard lazy-load data/positions-deputes/ppl/"{d.deputy_id.clone()}".json (backend V4.1)."
                                                </p>
                                            </div>
                                        </div>

                                        {move || match deputy_ppl_res.get() {
                                            None => view! {
                                                <div style="padding:.75rem;border:1px solid var(--bg-border);border-radius:8px;background:rgba(255,255,255,.01);color:var(--text-muted);font-size:.8rem;">
                                                    "Chargement des PPL associées..."
                                                </div>
                                            }.into_view(),
                                            Some(Err(e)) => view! {
                                                <div style="padding:.75rem;border:1px solid var(--danger);border-radius:8px;background:rgba(239,68,68,.08);color:var(--danger);font-size:.8rem;">
                                                    {format!("Erreur de chargement du shard PPL député : {}", e)}
                                                </div>
                                            }.into_view(),
                                            Some(Ok(None)) => view! {
                                                <div style="padding:.75rem;border:1px solid var(--bg-border);border-radius:8px;background:rgba(255,255,255,.01);color:var(--text-muted);font-size:.8rem;">
                                                    "Aucune donnée PPL associée pour ce député (shard absent ou non généré)."
                                                </div>
                                            }.into_view(),
                                            Some(Ok(Some(shard))) => {
                                                let shown_count = shard.items.len().min(20);
                                                let hidden_count = shard.items.len().saturating_sub(shown_count);
                                                view! {
                                                    <>
                                                        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(140px,1fr));gap:.6rem;margin-bottom:.75rem;">
                                                            <div style="padding:.55rem .65rem;border:1px solid var(--bg-border);border-radius:8px;">
                                                                <div style="font-size:.7rem;color:var(--text-muted);">"Entrées"</div>
                                                                <div style="font-weight:700;">{shard.total_entries.to_string()}</div>
                                                            </div>
                                                            <div style="padding:.55rem .65rem;border:1px solid var(--bg-border);border-radius:8px;">
                                                                <div style="font-size:.7rem;color:var(--text-muted);">"Auteur principal"</div>
                                                                <div style="font-weight:700;color:var(--success);">{shard.authored_count.to_string()}</div>
                                                            </div>
                                                            <div style="padding:.55rem .65rem;border:1px solid var(--bg-border);border-radius:8px;">
                                                                <div style="font-size:.7rem;color:var(--text-muted);">"Cosignature seule"</div>
                                                                <div style="font-weight:700;color:var(--accent);">{shard.cosigned_only_count.to_string()}</div>
                                                            </div>
                                                            <div style="padding:.55rem .65rem;border:1px solid var(--bg-border);border-radius:8px;">
                                                                <div style="font-size:.7rem;color:var(--text-muted);">"Groupe (snapshot)"</div>
                                                                <div style="font-weight:600;">{shard.group_label.clone().unwrap_or_else(|| "—".to_string())}</div>
                                                            </div>
                                                        </div>

                                                        {if shard.items.is_empty() {
                                                            view! {
                                                                <div style="padding:.75rem;border:1px solid var(--bg-border);border-radius:8px;background:rgba(255,255,255,.01);color:var(--text-muted);font-size:.8rem;">
                                                                    "Shard présent mais sans entrées."
                                                                </div>
                                                            }.into_view()
                                                        } else {
                                                            view! {
                                                                <div style="display:flex;flex-direction:column;gap:.45rem;">
                                                                    {shard.items.iter().take(20).cloned().map(render_deputy_ppl_item).collect_view()}
                                                                    {if hidden_count > 0 {
                                                                        view! {
                                                                            <div style="font-size:.74rem;color:var(--text-muted);padding:.15rem .1rem;">
                                                                                {format!("{} entrée(s) supplémentaires masquées (limite d'affichage 20).", hidden_count)}
                                                                            </div>
                                                                        }.into_view()
                                                                    } else {
                                                                        view! { <></> }.into_view()
                                                                    }}
                                                                </div>
                                                            }.into_view()
                                                        }}
                                                    </>
                                                }.into_view()
                                            }
                                        }}
                                    </div>

                                    // Répartition votes
                                    <div style=format!("margin-bottom:1.75rem;padding:1.25rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-left:3px solid {};border-radius:8px;", grp_color)>
                                        <h2 style="font-size:0.85rem;font-weight:600;margin:0 0 1rem 0;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);">
                                            "Répartition des positions (votes exprimés)"
                                        </h2>
                                        {if d.votes_exprimes > 0 {
                                            let total     = d.votes_exprimes as f64;
                                            let pour_pct  = d.pour_count    as f64 / total * 100.0;
                                            let contre_pct= d.contre_count  as f64 / total * 100.0;
                                            let abst_pct  = d.abst_count    as f64 / total * 100.0;
                                            view! {
                                                <div style="display:flex;gap:2px;height:20px;border-radius:4px;overflow:hidden;margin-bottom:0.75rem;"
                                                    role="img"
                                                    aria-label=format!("Pour {:.1}%, Contre {:.1}%, Abstention {:.1}%", pour_pct, contre_pct, abst_pct)>
                                                    <div style=format!("width:{pour_pct:.1}%;background:var(--success);")></div>
                                                    <div style=format!("width:{contre_pct:.1}%;background:var(--danger);")></div>
                                                    <div style=format!("width:{abst_pct:.1}%;background:var(--warning);")></div>
                                                </div>
                                                <div style="display:flex;gap:1.5rem;flex-wrap:wrap;font-size:0.78rem;">
                                                    <span>"■ Pour : "{format!("{:.1}%", pour_pct)}</span>
                                                    <span>"■ Contre : "{format!("{:.1}%", contre_pct)}</span>
                                                    <span>"■ Abstention : "{format!("{:.1}%", abst_pct)}</span>
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <p style="color:var(--text-muted);font-size:0.82rem;">
                                                    "Aucun vote enregistré sur la période."
                                                </p>
                                            }.into_view()
                                        }}
                                    </div>

                                    // Top dossiers
                                    {if !d.top_dossiers.is_empty() {
                                        view! {
                                            <div style="margin-bottom:1.75rem;">
                                                <h2 style="font-size:0.85rem;font-weight:600;margin:0 0 0.75rem 0;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);display:flex;align-items:center;gap:0.4rem;">
                                                    "Top dossiers — activité"
                                                    <InfoIcon text="Score = 1×votes + 2×amendements déposés + 1×interventions. Coefficient 2 sur les amendements : engagement actif de rédaction." />
                                                </h2>
                                                <div style=format!("background:var(--bg-secondary);border:1px solid var(--bg-border);border-left:3px solid {};border-radius:8px;overflow:hidden;", grp_color)>
                                                    <table class="data-table">
                                                        <thead>
                                                            <tr>
                                                                <th>"Dossier"</th>
                                                                <th>"Votes"</th>
                                                                <th>"Amd."</th>
                                                                <th>"Interv."</th>
                                                                <th>"Score"</th>
                                                            </tr>
                                                        </thead>
                                                        <tbody>
                                                            {d.top_dossiers.iter().map(|dos| view! {
                                                                <tr>
                                                                    <td style="max-width:360px;">
                                                                        <span style="font-size:0.7rem;color:var(--text-muted);font-family:monospace;display:block;">
                                                                            {dos.dossier_id.clone()}
                                                                        </span>
                                                                        <span style="font-size:0.82rem;">{dos.titre.clone()}</span>
                                                                    </td>
                                                                    <td>{dos.votes}</td>
                                                                    <td>{dos.amendements}</td>
                                                                    <td style="color:var(--text-muted);">{dos.interventions}</td>
                                                                    <td style="font-weight:600;color:var(--accent);">{dos.score}</td>
                                                                </tr>
                                                            }).collect_view()}
                                                        </tbody>
                                                    </table>
                                                </div>
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }}

                                    // Réseau de co-signatures (intra / hors groupe)
                                    <CosignNetworkSection d=d.clone() />

                                    // Traçabilité des chiffres
                                    <div style=format!("margin-bottom:1.75rem;padding:1rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-left:3px solid {};border-radius:10px;", grp_color)>
                                        <h2 style="font-size:0.82rem;font-weight:600;margin:0 0 .75rem 0;text-transform:uppercase;letter-spacing:.06em;color:var(--text-muted);">"Traçabilité & export"</h2>
                                        <div style="display:flex;flex-wrap:wrap;gap:.5rem;margin-bottom:.55rem;">
                                            <a href=dataset_json_url.clone() target="_blank" rel="noopener noreferrer" style="font-size:.75rem;padding:.35rem .65rem;border:1px solid var(--bg-border);border-radius:8px;background:rgba(255,255,255,0.01);color:var(--text-primary);text-decoration:none;">"JSON période ↗"</a>
                                            <a href=dataset_csv_url.clone() target="_blank" rel="noopener noreferrer" style="font-size:.75rem;padding:.35rem .65rem;border:1px solid var(--bg-border);border-radius:8px;background:rgba(255,255,255,0.01);color:var(--text-primary);text-decoration:none;">"CSV période ↗"</a>
                                            <a href=profile_url.clone() target="_blank" rel="noopener noreferrer" style="font-size:.75rem;padding:.35rem .65rem;border:1px solid var(--bg-border);border-radius:8px;background:rgba(255,255,255,0.01);color:var(--text-primary);text-decoration:none;">"Source AN ↗"</a>
                                        </div>
                                        <p style="margin:0;color:var(--text-muted);font-size:.78rem;line-height:1.5;">
                                            {format!("Période active: {} → {} · Scrutins éligibles: {} · Les comparaisons utilisent le dataset {} chargé côté navigateur.", d.period_start, d.period_end, d.scrutins_eligibles, current_period.label())}
                                        </p>
                                    </div>

                                    // Notice
                                    <div style=format!("padding:0.75rem 1rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-left:3px solid {};border-radius:6px;font-size:0.75rem;color:var(--text-muted);line-height:1.6;", grp_color)>
                                        "Ces indicateurs mesurent uniquement l'activité observable dans les données open data officielles. "
                                        "Ils ne reflètent pas le travail local, les réunions non publiques, les négociations informelles ni l'implication hors hémicycle. "
                                        <A href=app_href("/methodologie") attr:style="color:var(--accent);">"→ Lire la méthodologie complète"</A>
                                    </div>
                                </div>
                            }.into_view()
                        }
                    }
                }
            }}
        </div>
    }
}


#[component]
fn CosignNetworkSection(d: DeputeStats) -> impl IntoView {
    let accent_color = groupe_color(d.groupe_abrev.as_deref());
    let network = d.cosign_network.clone();
    let fallback_top = d.top_cosignataires.clone();

    if let Some(network) = network {
        let total = network.total_cosignatures;
        let unique = network.unique_cosignataires;
        let in_count = network.in_group_count;
        let out_count = network.out_group_count;
        let in_pct = pct_of(in_count, total);
        let out_pct = pct_of(out_count, total);
        let (badge_label, badge_color) = transversalite_badge(out_pct);
        let has_any = total > 0 || !network.in_group.is_empty() || !network.out_group_groups.is_empty();

        return view! {
            <div style="margin-bottom:1.75rem;">
                <h2 style="font-size:0.85rem;font-weight:600;margin:0 0 0.75rem 0;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);display:flex;align-items:center;gap:0.4rem;">
                    "Co-signatures (réseau)"
                    <InfoIcon text="Répartition des co-signatures d'amendements observées sur la période active, entre co-signataires du même groupe et hors groupe. Une co-signature n'implique pas nécessairement le même niveau d'implication que l'auteur." />
                </h2>

                <div style=format!("background:var(--bg-secondary);border:1px solid var(--bg-border);border-left:3px solid {};border-radius:10px;padding:1rem;", accent_color)>
                    {if has_any {
                        view! {
                            <>
                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(170px,1fr));gap:.6rem;margin-bottom:.85rem;">
                                    <MiniMetric label="Co-signatures" value=total.to_string() subtle="observées".to_string() />
                                    <MiniMetric label="Co-signataires uniques" value=unique.to_string() subtle="".to_string() />
                                    <MiniMetric label="Intra-groupe" value=format!("{:.1}%", in_pct) subtle=format!("{} occurrences", in_count) />
                                    <MiniMetric label="Hors groupe" value=format!("{:.1}%", out_pct) subtle=format!("{} occurrences", out_count) />
                                </div>

                                <div style="display:flex;align-items:center;justify-content:space-between;gap:.75rem;flex-wrap:wrap;margin-bottom:.55rem;">
                                    <div style="font-size:.78rem;color:var(--text-secondary);">"Répartition des co-signatures sur la période active"</div>
                                    <span style=format!("font-size:.72rem;padding:.18rem .5rem;border:1px solid var(--bg-border);border-radius:999px;color:{};", badge_color)>{badge_label}</span>
                                </div>

                                <div style="display:flex;height:12px;border-radius:999px;overflow:hidden;border:1px solid var(--bg-border);background:rgba(255,255,255,.02);margin-bottom:.8rem;">
                                    <div style=format!("width:{:.4}%;background:var(--success);", in_pct) title=format!("Intra-groupe: {:.1}%", in_pct)></div>
                                    <div style=format!("width:{:.4}%;background:var(--warning);", out_pct) title=format!("Hors groupe: {:.1}%", out_pct)></div>
                                </div>

                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:.75rem;">
                                    <details open style="border:1px solid var(--bg-border);border-radius:8px;background:rgba(255,255,255,.01);padding:.2rem .6rem .55rem .6rem;">
                                        <summary style="cursor:pointer;font-size:.82rem;font-weight:600;padding:.35rem 0;">
                                            {format!("Dans le groupe ({} · {:.1}%)", in_count, in_pct)}
                                        </summary>
                                        {if network.in_group.is_empty() {
                                            view! { <p style="margin:.35rem 0 0 0;font-size:.78rem;color:var(--text-muted);">"Aucune co-signature intra-groupe observée sur cette période."</p> }.into_view()
                                        } else {
                                            view! {
                                                <div style="display:flex;flex-direction:column;gap:.35rem;margin-top:.35rem;">
                                                    {network.in_group.iter().map(|peer| view! {
                                                        <PeerRow peer=peer.clone() />
                                                    }).collect_view()}
                                                </div>
                                            }.into_view()
                                        }}
                                    </details>

                                    <details open style="border:1px solid var(--bg-border);border-radius:8px;background:rgba(255,255,255,.01);padding:.2rem .6rem .55rem .6rem;">
                                        <summary style="cursor:pointer;font-size:.82rem;font-weight:600;padding:.35rem 0;">
                                            {format!("Hors groupe ({} · {:.1}%)", out_count, out_pct)}
                                        </summary>
                                        {if network.out_group_groups.is_empty() {
                                            view! { <p style="margin:.35rem 0 0 0;font-size:.78rem;color:var(--text-muted);">"Aucune co-signature hors groupe observée sur cette période."</p> }.into_view()
                                        } else {
                                            view! {
                                                <div style="display:flex;flex-direction:column;gap:.45rem;margin-top:.35rem;">
                                                    {network.out_group_groups.iter().map(|bucket| {
                                                        let bucket_label = format_group_bucket_label(bucket.groupe_abrev.as_deref(), bucket.groupe_nom.as_deref());
                                                        let bucket_pct = pct_of(bucket.count_total, out_count);
                                                        view! {
                                                            <details style="border:1px solid var(--bg-border);border-radius:8px;padding:.2rem .55rem;background:rgba(255,255,255,.01);">
                                                                <summary style="cursor:pointer;padding:.28rem 0;display:flex;align-items:center;justify-content:space-between;gap:.5rem;flex-wrap:wrap;">
                                                                    <span style="font-size:.78rem;font-weight:600;">{bucket_label}</span>
                                                                    <span style="font-size:.72rem;color:var(--text-secondary);">{format!("{} · {:.1}% du hors-groupe", bucket.count_total, bucket_pct)}</span>
                                                                </summary>
                                                                <div style="display:flex;flex-direction:column;gap:.3rem;margin:.25rem 0 .2rem 0;">
                                                                    {bucket.members.iter().map(|peer| view! { <PeerRow peer=peer.clone() /> }).collect_view()}
                                                                </div>
                                                            </details>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_view()
                                        }}
                                    </details>
                                </div>

                                <p style="margin:.75rem 0 0 0;color:var(--text-muted);font-size:.74rem;line-height:1.5;">
                                    "Mesure basée sur les co-signatures d’amendements observées sur la période active. Une co-signature n’implique pas nécessairement le même niveau d’implication que l’auteur."
                                </p>
                            </>
                        }.into_view()
                    } else {
                        view! {
                            <p style="margin:0;font-size:.8rem;color:var(--text-muted);">
                                "Aucune co-signature exploitable sur cette période."
                            </p>
                        }.into_view()
                    }}
                </div>
            </div>
        }.into_view();
    }

    if !fallback_top.is_empty() {
        return view! {
            <div style="margin-bottom:1.75rem;">
                <h2 style="font-size:0.85rem;font-weight:600;margin:0 0 0.75rem 0;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);display:flex;align-items:center;gap:0.4rem;">
                    "Co-signatures (réseau)"
                    <InfoIcon text="Fallback d’affichage : détail réseau non disponible dans ce dataset, affichage des co-signataires les plus fréquents." />
                </h2>
                <div style=format!("background:var(--bg-secondary);border:1px solid var(--bg-border);border-left:3px solid {};border-radius:8px;overflow:hidden;", accent_color)>
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>"Député"</th>
                                <th>"Groupe"</th>
                                <th>"Co-signatures"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {fallback_top.iter().cloned().map(|c| {
                                let deputy_id = c.deputy_id.clone();
                                let deputy_href = app_href(&format!("/depute/{}", deputy_id));
                                let full_name = format!("{} {}", c.prenom, c.nom);
                                let groupe = c.groupe_abrev.unwrap_or_else(|| "—".to_string());
                                let co_signed_count = c.co_signed_count;
                                view! {
                                    <tr>
                                        <td>
                                            <A href=deputy_href attr:style="color:var(--text-primary);text-decoration:none;">
                                                {full_name}
                                            </A>
                                            <div style="font-size:.7rem;color:var(--text-muted);font-family:monospace;">{deputy_id}</div>
                                        </td>
                                        <td>{groupe}</td>
                                        <td style="font-weight:600;color:var(--accent);">{co_signed_count}</td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
            </div>
        }.into_view();
    }

    view! {
        <div style="margin-bottom:1.75rem;">
            <h2 style="font-size:0.85rem;font-weight:600;margin:0 0 0.75rem 0;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);display:flex;align-items:center;gap:0.4rem;">
                "Co-signatures (réseau)"
                <InfoIcon text="Répartition des co-signatures d'amendements observées sur la période active, intra-groupe vs hors groupe." />
            </h2>
            <div style=format!("background:var(--bg-secondary);border:1px solid var(--bg-border);border-left:3px solid {};border-radius:10px;padding:1rem;", accent_color)>
                <p style="margin:0;font-size:.8rem;color:var(--text-muted);">"Aucune co-signature exploitable sur cette période."</p>
            </div>
        </div>
    }
    .into_view()
}

#[component]
fn MiniMetric(label: &'static str, value: String, #[prop(optional)] subtle: Option<String>) -> impl IntoView {
    view! {
        <div style="padding:.6rem .7rem;border:1px solid var(--bg-border);border-radius:8px;background:rgba(255,255,255,.01);">
            <div style="font-size:.72rem;color:var(--text-secondary);margin-bottom:.15rem;">{label}</div>
            <div style="font-size:.95rem;font-weight:700;color:var(--text-primary);line-height:1.15;">{value}</div>
            {subtle.map(|txt| view! {
                <div style="font-size:.7rem;color:var(--text-muted);margin-top:.18rem;">{txt}</div>
            })}
        </div>
    }
}

#[component]
fn PeerRow(peer: CosignPeer) -> impl IntoView {
    let group = peer.groupe_abrev.clone().unwrap_or_else(|| "—".to_string());
    view! {
        <div style="display:flex;align-items:center;justify-content:space-between;gap:.5rem;padding:.35rem .45rem;border:1px solid rgba(255,255,255,.03);border-radius:6px;background:rgba(255,255,255,.01);">
            <div style="min-width:0;">
                <A href=app_href(&format!("/depute/{}", peer.deputy_id)) attr:style="color:var(--text-primary);text-decoration:none;font-size:.78rem;">
                    {format!("{} {}", peer.prenom, peer.nom)}
                </A>
                <div style="font-size:.68rem;color:var(--text-muted);font-family:monospace;">{peer.deputy_id.clone()}</div>
            </div>
            <div style="display:flex;align-items:center;gap:.45rem;flex-shrink:0;">
                <span style="font-size:.68rem;padding:.15rem .4rem;border:1px solid var(--bg-border);border-radius:999px;color:var(--text-secondary);">{group}</span>
                <span style="font-size:.8rem;font-weight:700;color:var(--accent);min-width:2.2rem;text-align:right;">{peer.count}</span>
            </div>
        </div>
    }
}

fn pct_of(part: u32, total: u32) -> f64 {
    if total == 0 { 0.0 } else { (part as f64 / total as f64) * 100.0 }
}

fn transversalite_badge(out_pct: f64) -> (&'static str, &'static str) {
    if out_pct > 25.0 {
        ("Transpartisan", "var(--warning)")
    } else if out_pct >= 10.0 {
        ("Mixte", "var(--accent)")
    } else {
        ("Très intra-groupe", "var(--success)")
    }
}

fn format_group_bucket_label(abrev: Option<&str>, nom: Option<&str>) -> String {
    match (abrev, nom) {
        (Some(a), Some(n)) if !a.is_empty() && !n.is_empty() => format!("{} — {}", a, n),
        (Some(a), _) if !a.is_empty() => a.to_string(),
        (_, Some(n)) if !n.is_empty() => n.to_string(),
        _ => "Groupe non renseigné".to_string(),
    }
}


#[derive(Debug, Clone)]
struct PeriodSnapshotMini {
    period: Period,
    participation_rate: f64,
    amd_authored: u32,
    amd_adopted: u32,
    scrutins_eligibles: u32,
}

#[derive(Debug, Clone, Default)]
struct Benchmarks {
    participation_median: Option<f64>,
    participation_group_avg: Option<f64>,
    amd_authored_median: Option<f64>,
    amd_authored_group_avg: Option<f64>,
    amd_adopted_median: Option<f64>,
    amd_adopted_group_avg: Option<f64>,
    amd_adoption_rate_median: Option<f64>,
    amd_adoption_rate_group_avg: Option<f64>,
}

#[component]
fn BenchmarkRow(
    label: &'static str,
    value: f64,
    median: Option<f64>,
    group_avg: Option<f64>,
    #[prop(optional)] is_percent: bool,
) -> impl IntoView {
    let value_label = if is_percent { fmt_pct(value) } else { fmt_number(value) };
    let median_delta = median.map(|m| value - m);
    let group_delta = group_avg.map(|g| value - g);

    view! {
        <div style="padding:.65rem .75rem;border:1px solid var(--bg-border);border-radius:8px;background:rgba(255,255,255,.01);">
            <div style="display:flex;align-items:center;justify-content:space-between;gap:.75rem;flex-wrap:wrap;">
                <div style="font-size:.78rem;color:var(--text-secondary);">{label}</div>
                <div style="font-size:.88rem;font-weight:600;color:var(--text-primary);">{value_label}</div>
            </div>
            <div style="display:flex;gap:.4rem;flex-wrap:wrap;margin-top:.4rem;">
                <DeltaChip label="vs médiane" delta=median_delta is_percent=is_percent />
                <DeltaChip label="vs groupe" delta=group_delta is_percent=is_percent />
            </div>
        </div>
    }
}

#[component]
fn DeltaChip(
    label: &'static str,
    delta: Option<f64>,
    #[prop(optional)] is_percent: bool,
) -> impl IntoView {
    let (txt, color) = match delta {
        Some(d) => {
            let sign = if d > 0.0 { "+" } else { "" };
            let v = if is_percent { format!("{sign}{:.1} pts", d * 100.0) } else { format!("{sign}{:.1}", d) };
            let color = if d > 0.0 {
                "var(--success)"
            } else if d < 0.0 {
                "var(--danger)"
            } else {
                "var(--text-muted)"
            };
            (format!("{label}: {v}"), color)
        }
        None => (format!("{label}: —"), "var(--text-muted)"),
    };

    view! {
        <span style=format!("font-size:.72rem;padding:.18rem .45rem;border:1px solid var(--bg-border);border-radius:999px;color:{color};")>
            {txt}
        </span>
    }
}

fn render_deputy_ppl_item(item: DeputyPplItemSummary) -> View {
    let title_meta = {
        let mut parts = Vec::new();
        if let Some(n) = &item.number {
            if !n.trim().is_empty() {
                parts.push(format!("n°{}", n));
            }
        }
        if let Some(d) = &item.deposit_date {
            if !d.trim().is_empty() {
                parts.push(d.clone());
            }
        }
        parts.join(" · ")
    };

    let title_text = item.title.clone();
    let source_url = item
        .source_url
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())
        .map(|u| u.to_string());

    let title_view = if let Some(url) = source_url.clone() {
        view! {
            <div style="display:flex;align-items:flex-start;gap:.35rem;flex-wrap:wrap;">
                <a href=url.clone() target="_blank" rel="noopener noreferrer" style="color:var(--text-primary);text-decoration:none;font-weight:600;line-height:1.35;">
                    {title_text}
                </a>
                <a href=url target="_blank" rel="noopener noreferrer" style="font-size:.7rem;color:var(--text-muted);text-decoration:none;border:1px solid var(--bg-border);border-radius:999px;padding:.05rem .35rem;line-height:1.2;">
                    "AN ↗"
                </a>
            </div>
        }.into_view()
    } else {
        view! { <span style="font-weight:600;line-height:1.35;">{title_text}</span> }.into_view()
    };

    view! {
        <div style="padding:.65rem .75rem;border:1px solid var(--bg-border);border-radius:8px;background:rgba(255,255,255,.01);display:flex;flex-direction:column;gap:.25rem;">
            <div style="display:flex;align-items:flex-start;justify-content:space-between;gap:.65rem;flex-wrap:wrap;">
                <div style="min-width:240px;flex:1;display:flex;flex-direction:column;gap:.15rem;">
                    {title_view}
                    {if !title_meta.is_empty() {
                        view! { <span style="font-size:.73rem;color:var(--text-muted);">{title_meta}</span> }.into_view()
                    } else {
                        view! { <></> }.into_view()
                    }}
                    <span style="font-size:.7rem;color:var(--text-muted);font-family:monospace;">{item.ppl_id.clone()}</span>
                </div>
                <div style="display:flex;align-items:center;gap:.35rem;flex-wrap:wrap;">
                    {if item.is_author {
                        view! {
                            <span style="display:inline-flex;align-items:center;padding:.14rem .45rem;border-radius:999px;font-size:.68rem;font-weight:600;background:rgba(34,197,94,.12);color:#22c55e;border:1px solid rgba(34,197,94,.25);">
                                "Auteur"
                            </span>
                        }.into_view()
                    } else { view! { <></> }.into_view() }}
                    {if item.is_cosigner {
                        view! {
                            <span style="display:inline-flex;align-items:center;padding:.14rem .45rem;border-radius:999px;font-size:.68rem;font-weight:600;background:rgba(59,130,246,.12);color:#60a5fa;border:1px solid rgba(59,130,246,.25);">
                                "Cosignataire"
                            </span>
                        }.into_view()
                    } else { view! { <></> }.into_view() }}
                </div>
            </div>
        </div>
    }.into_view()
}

fn collect_period_snapshots(store: &crate::store::AppStore, deputy_id: &str) -> Vec<PeriodSnapshotMini> {
    [Period::P30, Period::P180, Period::Leg]
        .into_iter()
        .filter_map(|p| {
            store.find_depute(p, deputy_id).map(|d| PeriodSnapshotMini {
                period: p,
                participation_rate: d.participation_rate,
                amd_authored: d.amd_authored,
                amd_adopted: d.amd_adopted,
                scrutins_eligibles: d.scrutins_eligibles,
            })
        })
        .collect()
}

fn compute_benchmarks(target: &DeputeStats, rows: &[DeputeStats]) -> Benchmarks {
    let same_group: Vec<&DeputeStats> = rows
        .iter()
        .filter(|r| r.groupe_abrev == target.groupe_abrev && r.groupe_nom == target.groupe_nom)
        .collect();

    Benchmarks {
        participation_median: median_of(rows.iter().map(|r| r.participation_rate)),
        participation_group_avg: mean_of(same_group.iter().map(|r| r.participation_rate)),
        amd_authored_median: median_of(rows.iter().map(|r| r.amd_authored as f64)),
        amd_authored_group_avg: mean_of(same_group.iter().map(|r| r.amd_authored as f64)),
        amd_adopted_median: median_of(rows.iter().map(|r| r.amd_adopted as f64)),
        amd_adopted_group_avg: mean_of(same_group.iter().map(|r| r.amd_adopted as f64)),
        amd_adoption_rate_median: median_of(rows.iter().filter_map(|r| r.amd_adoption_rate)),
        amd_adoption_rate_group_avg: mean_of(same_group.iter().filter_map(|r| r.amd_adoption_rate)),
    }
}

fn mean_of<I>(iter: I) -> Option<f64>
where
    I: Iterator<Item = f64>,
{
    let mut n = 0u32;
    let mut sum = 0.0f64;
    for v in iter {
        n += 1;
        sum += v;
    }
    if n == 0 { None } else { Some(sum / n as f64) }
}

fn median_of<I>(iter: I) -> Option<f64>
where
    I: Iterator<Item = f64>,
{
    let mut v: Vec<f64> = iter.filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = v.len() / 2;
    if v.len() % 2 == 1 {
        Some(v[mid])
    } else {
        Some((v[mid - 1] + v[mid]) / 2.0)
    }
}

fn fmt_number(value: f64) -> String {
    if (value - value.round()).abs() < f64::EPSILON {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.1}")
    }
}


fn format_mandat_episodes_summary(episodes: &[MandatAssembleeEpisode]) -> String {
    episodes
        .iter()
        .map(|ep| format_mandat_episode(ep))
        .collect::<Vec<_>>()
        .join(" • ")
}

fn format_mandat_episode(ep: &MandatAssembleeEpisode) -> String {
    let fin = ep
        .date_fin
        .map(|d| d.to_string())
        .unwrap_or_else(|| "en cours".to_string());
    format!("{} → {}", ep.date_debut, fin)
}

fn an_profile_url(deputy_id: &str) -> String {
    format!("https://www.assemblee-nationale.fr/dyn/deputes/{deputy_id}")
}

fn an_photo_url(deputy_id: &str) -> Option<String> {
    // Les pages députés "dyn" pointent vers des photos avec identifiant numérique (sans préfixe PA)
    // Exemple observé: /dyn/static/tribun/17/photos/carre/1008.jpg pour PA1008.
    let numeric_id: String = deputy_id.chars().filter(|c| c.is_ascii_digit()).collect();
    if numeric_id.is_empty() {
        return None;
    }

    Some(format!(
        "https://www.assemblee-nationale.fr/dyn/static/tribun/17/photos/carre/{numeric_id}.jpg"
    ))
}

fn initials(prenom: &str, nom: &str) -> String {
    let p = prenom.chars().find(|c| c.is_alphabetic()).unwrap_or('?');
    let n = nom.chars().find(|c| c.is_alphabetic()).unwrap_or('?');
    let p = p.to_uppercase().collect::<String>();
    let n = n.to_uppercase().collect::<String>();
    format!("{}{}", p, n)
}

fn normalize_external_url(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return String::new();
    }
    if raw.starts_with("http://") || raw.starts_with("https://") || raw.starts_with("mailto:") {
        raw.to_string()
    } else {
        format!("https://{}", raw.trim_start_matches('/'))
    }
}

fn dept_circo_label(dept: Option<&str>, circo: Option<&str>) -> Option<String> {
    match (dept, circo) {
        (Some(d), Some(c)) if !d.is_empty() && !c.is_empty() => Some(format!("{} — Circonscription n°{}", d, c)),
        (Some(d), _) if !d.is_empty() => Some(d.to_string()),
        _ => None,
    }
}

fn display_host(url: &str) -> String {
    let s = url.trim_start_matches("https://").trim_start_matches("http://");
    s.trim_end_matches('/').to_string()
}

fn extra_site_label_from_source(src: &SiteWebSource) -> String {
    let platform = src
        .type_libelle
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let account = src.val_elec.trim();

    match (platform, account.is_empty()) {
        (Some(p), false) => format!("Présent(e) sur {} : {}", p, account),
        (Some(p), true) => format!("Présent(e) sur {}", p),
        (None, false) => {
            if let Some(url) = src.url.as_deref() {
                extra_site_label(url)
            } else {
                account.to_string()
            }
        }
        (None, true) => src
            .url
            .as_deref()
            .map(extra_site_label)
            .unwrap_or_else(|| "Autre site".to_string()),
    }
}

fn extra_site_label(url: &str) -> String {
    let no_scheme = url
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("//");
    let no_scheme = no_scheme.split('#').next().unwrap_or(no_scheme);
    let no_scheme = no_scheme.split('?').next().unwrap_or(no_scheme);

    let mut parts = no_scheme.splitn(2, '/');
    let host_raw = parts.next().unwrap_or("").trim().to_lowercase();
    let path = parts.next().unwrap_or("").trim();

    let host = host_raw
        .strip_prefix("www.")
        .or_else(|| host_raw.strip_prefix("m."))
        .unwrap_or(&host_raw);

    let mut segs = path.split('/').filter(|s| !s.is_empty());
    let seg1 = segs.next().unwrap_or("");
    let seg2 = segs.next().unwrap_or("");
    let clean = |s: &str| -> String { s.trim_start_matches('@').to_string() };

    match host {
        "linkedin.com" | "fr.linkedin.com" => match seg1 {
            "in" if !seg2.is_empty() => format!("LinkedIn · {}", clean(seg2)),
            "company" if !seg2.is_empty() => format!("LinkedIn · {}", clean(seg2)),
            _ => "LinkedIn".to_string(),
        },
        "x.com" | "twitter.com" if !seg1.is_empty() => format!("X/Twitter · @{}", clean(seg1)),
        "x.com" | "twitter.com" => "X/Twitter".to_string(),
        "facebook.com" if !seg1.is_empty() => format!("Facebook · {}", clean(seg1)),
        "facebook.com" => "Facebook".to_string(),
        "instagram.com" if !seg1.is_empty() => format!("Instagram · @{}", clean(seg1)),
        "instagram.com" => "Instagram".to_string(),
        "github.com" if !seg1.is_empty() => format!("GitHub · {}", clean(seg1)),
        "github.com" => "GitHub".to_string(),
        "youtube.com" | "youtu.be" => "YouTube".to_string(),
        "tiktok.com" if !seg1.is_empty() => format!("TikTok · {}", clean(seg1)),
        "tiktok.com" => "TikTok".to_string(),
        "wikipedia.org" | "fr.wikipedia.org" => "Wikipédia".to_string(),
        _ if !host.is_empty() => format!("Présent sur {}", host),
        _ => "Autre site".to_string(),
    }
}
