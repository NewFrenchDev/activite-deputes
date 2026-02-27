use leptos::*;
use leptos_router::{A, use_query_map};

use crate::api::{fetch_group_ppl_group_shard, fetch_group_ppl_index, inferred_github_repo_urls};
use crate::models::{GroupPplGroupIndexEntry, GroupPplItemSummary, SignerPreviewEntry};
use crate::utils::{matches_search, normalize_search, app_href};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RelationFilter {
    All,
    AuthorOnly,
    CosignedOnly,
}

impl RelationFilter {
    fn from_query(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "author" | "auteur" | "authored" => Self::AuthorOnly,
            "cosigned" | "cosigner" | "cosigne" | "cosignataire" => Self::CosignedOnly,
            _ => Self::All,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::All => "all",
            Self::AuthorOnly => "author",
            Self::CosignedOnly => "cosigned",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::All => "Tous",
            Self::AuthorOnly => "Auteur présent",
            Self::CosignedOnly => "Cosignature uniquement",
        }
    }
}

#[component]
pub fn PositionsGroupesPage() -> impl IntoView {
    let query_map = use_query_map();

    let index_res = create_resource(|| (), |_| fetch_group_ppl_index());

    let (selected_group_id, set_selected_group_id) = create_signal::<Option<String>>(None);
    let (selected_group_file, set_selected_group_file) = create_signal::<Option<String>>(None);
    let (search, set_search) = create_signal(String::new());
    let (relation_filter, set_relation_filter) = create_signal(RelationFilter::All);
    let (include_unknown, set_include_unknown) = create_signal(false);
    let (limit_rows, set_limit_rows) = create_signal(100usize);
    let issue_url = inferred_github_repo_urls().map(|(_, issues)| issues);

    // Initialise depuis les query params et choisit un groupe par défaut.
    create_effect(move |_| {
        let idx = match index_res.get() {
            Some(Ok(idx)) => idx,
            _ => return,
        };

        let mut q_group: Option<String> = None;
        let mut q_search: Option<String> = None;
        let mut q_rel: Option<String> = None;
        let mut q_unknown: Option<String> = None;
        query_map.with(|q| {
            q_group = q.get("group").cloned();
            q_search = q.get("q").cloned();
            q_rel = q.get("rel").cloned();
            q_unknown = q.get("unknown").cloned();
        });

        if let Some(q) = q_search.clone() {
            if search.get_untracked().is_empty() && !q.trim().is_empty() {
                set_search.set(q);
            }
        }
        if let Some(r) = q_rel.clone() {
            if relation_filter.get_untracked() == RelationFilter::All {
                set_relation_filter.set(RelationFilter::from_query(&r));
            }
        }
        if let Some(u) = q_unknown.clone() {
            let val = matches!(u.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "oui");
            if include_unknown.get_untracked() != val {
                set_include_unknown.set(val);
            }
        }

        let current_id = selected_group_id.get_untracked();
        let current_still_exists = current_id
            .as_ref()
            .map(|id| idx.groups.iter().any(|g| &g.group_id == id))
            .unwrap_or(false);

        if current_still_exists {
            return;
        }

        let requested = q_group.as_deref().map(str::trim).filter(|s| !s.is_empty());
        let mut target: Option<&GroupPplGroupIndexEntry> = None;

        if let Some(req) = requested {
            let req_norm = normalize_search(req);
            target = idx.groups.iter().find(|g| {
                g.group_id.eq_ignore_ascii_case(req)
                    || normalize_search(&g.group_label) == req_norm
                    || g.file.eq_ignore_ascii_case(req)
            });
        }

        if target.is_none() {
            target = idx.groups.iter().find(|g| g.group_id != "INCONNU");
        }
        if target.is_none() {
            target = idx.groups.first();
        }

        if let Some(g) = target {
            set_selected_group_id.set(Some(g.group_id.clone()));
            set_selected_group_file.set(Some(g.file.clone()));
        }
    });

    // Quand le groupe sélectionné change, retrouve son fichier shard.
    let index_res_for_sync = index_res.clone();
    create_effect(move |_| {
        let selected = selected_group_id.get();
        let idx = match index_res_for_sync.get() {
            Some(Ok(idx)) => idx,
            _ => return,
        };
        match selected {
            Some(id) => {
                if let Some(g) = idx.groups.iter().find(|g| g.group_id == id) {
                    let next_file = Some(g.file.clone());
                    if selected_group_file.get_untracked() != next_file {
                        set_selected_group_file.set(next_file);
                    }
                }
            }
            None => {
                if selected_group_file.get_untracked().is_some() {
                    set_selected_group_file.set(None);
                }
            }
        }
    });

    let shard_res = create_resource(
        move || selected_group_file.get(),
        |file_opt| async move {
            match file_opt {
                Some(file) => fetch_group_ppl_group_shard(&file).await.map(Some),
                None => Ok(None),
            }
        },
    );

    let selected_group_label = create_memo(move |_| {
        let idx = match index_res.get() {
            Some(Ok(idx)) => idx,
            _ => return None,
        };
        let sel = selected_group_id.get()?;
        idx.groups
            .iter()
            .find(|g| g.group_id == sel)
            .map(|g| g.group_label.clone())
    });

    let filtered_items = create_memo(move |_| {
        let shard = match shard_res.get() {
            Some(Ok(Some(shard))) => shard,
            _ => return Vec::<GroupPplItemSummary>::new(),
        };

        let q = search.get();
        let q_norm = q.trim().to_string();
        let rel = relation_filter.get();
        let include_unknown_now = include_unknown.get();
        let shard_group_id = shard.group_id.clone();

        if !include_unknown_now && shard_group_id == "INCONNU" {
            // Le bucket inconnu est lourd et peu actionnable en V1, on le masque par défaut.
            // Si l'utilisateur l'a explicitement choisi, on l'affichera quand include_unknown=true.
            return Vec::new();
        }

        let mut out = Vec::with_capacity(shard.items.len());
        for item in shard.items.into_iter() {

            let rel_ok = match rel {
                RelationFilter::All => true,
                RelationFilter::AuthorOnly => item.has_author,
                RelationFilter::CosignedOnly => !item.has_author,
            };
            if !rel_ok {
                continue;
            }

            if !q_norm.is_empty() {
                let mut hay = item.title.clone();
                if let Some(n) = &item.number {
                    hay.push(' ');
                    hay.push_str(n);
                }
                if !matches_search(&hay, &q_norm) {
                    continue;
                }
            }
            out.push(item);
        }
        out
    });

    let displayed_items = create_memo(move |_| {
        let limit = limit_rows.get();
        let mut items = filtered_items.get();
        if items.len() > limit {
            items.truncate(limit);
        }
        items
    });

    let hidden_unknown_count = create_memo(move |_| {
        match index_res.get() {
            Some(Ok(idx)) if !include_unknown.get() => idx
                .groups
                .iter()
                .find(|g| g.group_id == "INCONNU")
                .map(|g| g.ppl_count)
                .unwrap_or(0),
            _ => 0,
        }
    });

    view! {
        <div class="reveal" style="display:flex;flex-direction:column;gap:1rem;">
            <section style="padding:1rem;background:linear-gradient(180deg, rgba(34,197,94,.05), rgba(34,197,94,.015));border:1px solid var(--bg-border);border-radius:12px;">
                <div style="display:flex;align-items:flex-start;justify-content:space-between;gap:1rem;flex-wrap:wrap;">
                    <div style="max-width:930px;">
                        <h1 style="margin:0 0 .35rem 0;font-size:1.15rem;font-weight:700;">"Positions des groupes via les propositions de loi"</h1>
                        <p style="margin:0;color:var(--text-muted);font-size:.83rem;line-height:1.45;">
                            "V1 frontend : lecture des agrégats PPL par groupe générés côté pipeline. "
                            "Le backend V4 filtre les propositions d'origine Assemblée pour limiter les faux "
                            "INCONNU (cas Sénat / navette). Les amendements viendront ensuite."
                        </p>
                        {move || {
                            let hidden = hidden_unknown_count.get();
                            if hidden > 0 {
                                view! {
                                    <p style="margin:.45rem 0 0 0;color:var(--text-muted);font-size:.74rem;">
                                        {format!("Le bucket INCONNU est masqué par défaut ({} entrées). Activez le toggle pour l'inspecter.", hidden)}
                                    </p>
                                }.into_view()
                            } else {
                                view! { <></> }.into_view()
                            }
                        }}
                    </div>
                    <div style="display:flex;align-items:center;gap:.5rem;flex-wrap:wrap;">
                        <A href=app_href("/methodologie") class="btn" attr:style="text-decoration:none;">"Méthode"</A>
                        {match issue_url.clone() {
                            Some(url) => view! {
                                <a href=url target="_blank" rel="noopener noreferrer" class="btn" style="text-decoration:none;">"Feedback ↗"</a>
                            }.into_view(),
                            None => view! {
                                <A href=app_href("/methodologie#retours") class="btn" attr:style="text-decoration:none;">"Feedback"</A>
                            }.into_view(),
                        }}
                    </div>
                </div>
            </section>

            {move || match index_res.get() {
                None => view! {
                    <div style="padding:1rem;border:1px solid var(--bg-border);border-radius:10px;background:var(--bg-secondary);color:var(--text-muted);">
                        "Chargement de l'index PPL par groupe..."
                    </div>
                }.into_view(),
                Some(Err(e)) => view! {
                    <div style="padding:1rem;border:1px solid var(--danger);border-radius:10px;background:rgba(239,68,68,.08);color:var(--danger);">
                        {format!("Erreur de chargement de data/positions-groupes/ppl/index.json : {}", e)}
                    </div>
                }.into_view(),
                Some(Ok(index)) => {
                    if index.groups.is_empty() {
                        view! {
                            <div style="padding:1rem;border:1px solid var(--bg-border);border-radius:10px;background:var(--bg-secondary);color:var(--text-muted);">
                                "Aucune donnée PPL par groupe disponible (index vide). Vérifie la génération backend et les fichiers data/positions-groupes/ppl/*.json."
                            </div>
                        }.into_view()
                    } else {
                        let known_groups = index.groups.iter().filter(|g| g.group_id != "INCONNU").count();
                        let selected_id_for_header = selected_group_id.get();
                        let selected_entry = selected_id_for_header
                            .as_ref()
                            .and_then(|id| index.groups.iter().find(|g| &g.group_id == id));
                        let index_for_select = index.clone();
                        let index_for_chips = index.clone();

                        view! {
                            <>
                                <div style="padding:.7rem .9rem;border:1px solid var(--bg-border);border-radius:10px;background:rgba(255,255,255,.015);display:flex;align-items:center;justify-content:space-between;gap:.75rem;flex-wrap:wrap;">
                                    <div style="font-size:.77rem;color:var(--text-muted);">
                                        "Dernière génération de l’index PPL groupes : "
                                        <strong style="color:var(--text-secondary);">{index.generated_at.clone()}</strong>
                                    </div>
                                    <div style="font-size:.73rem;color:var(--text-muted);">"Source: data/positions-groupes/ppl/index.json"</div>
                                </div>
                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:.75rem;">
                                    <MiniStatCard label="Groupes (index)" value=index.total_groups.to_string() sub=format!("{} hors bucket INCONNU", known_groups) />
                                    <MiniStatCard label="Liens groupe↔PPL" value=index.total_ppl_links.to_string() sub="entrées agrégées (groupe,ppl)".to_string() />
                                    <MiniStatCard label="PPL uniques" value=index.total_unique_ppl.to_string() sub="dans le périmètre exporté".to_string() />
                                    <MiniStatCard label="Groupe sélectionné" value=selected_group_label.get().unwrap_or_else(|| "—".to_string()) sub=selected_entry.map(|g| format!("{} entrées", g.ppl_count)).unwrap_or_else(|| "choisissez un groupe".to_string()) />
                                </div>

                                <section style="display:grid;grid-template-columns:minmax(260px, 360px) 1fr;gap:1rem;align-items:start;">
                                    <div style="display:flex;flex-direction:column;gap:.85rem;position:sticky;top:72px;">
                                        <div style="padding:.9rem;border:1px solid var(--bg-border);border-radius:10px;background:var(--bg-secondary);display:flex;flex-direction:column;gap:.7rem;">
                                            <div>
                                                <label style="display:block;font-size:.76rem;color:var(--text-muted);margin-bottom:.35rem;">"Groupe"</label>
                                                <select
                                                    class="input"
                                                    style="width:100%;"
                                                    on:change=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        if value.trim().is_empty() {
                                                            set_selected_group_id.set(None);
                                                        } else {
                                                            set_selected_group_id.set(Some(value));
                                                        }
                                                    }
                                                >
                                                    {index_for_select.groups.iter().filter(|g| include_unknown.get() || g.group_id != "INCONNU").map(|g| {
                                                        let label = if g.group_id == "INCONNU" {
                                                            format!("{} ({}) ⚠", g.group_label, g.ppl_count)
                                                        } else {
                                                            format!("{} ({})", g.group_label, g.ppl_count)
                                                        };
                                                        let gid = g.group_id.clone();
                                                        let gid_sel = g.group_id.clone();
                                                        view! { <option value={gid} selected=move || selected_group_id.get().as_deref() == Some(gid_sel.as_str())>{label}</option> }
                                                    }).collect_view()}
                                                </select>
                                            </div>

                                            <div>
                                                <label style="display:block;font-size:.76rem;color:var(--text-muted);margin-bottom:.35rem;">"Recherche PPL"</label>
                                                <input
                                                    class="input"
                                                    type="text"
                                                    placeholder="Titre ou numéro"
                                                    prop:value=move || search.get()
                                                    on:input=move |ev| set_search.set(event_target_value(&ev))
                                                    style="width:100%;"
                                                />
                                            </div>

                                            <div>
                                                <label style="display:block;font-size:.76rem;color:var(--text-muted);margin-bottom:.35rem;">"Relation"</label>
                                                <select
                                                    class="input"
                                                    style="width:100%;"
                                                    on:change=move |ev| {
                                                        let v = event_target_value(&ev);
                                                        set_relation_filter.set(RelationFilter::from_query(&v));
                                                    }
                                                    prop:value=move || relation_filter.get().as_str().to_string()
                                                >
                                                    <option value="all">{RelationFilter::All.label()}</option>
                                                    <option value="author">{RelationFilter::AuthorOnly.label()}</option>
                                                    <option value="cosigned">{RelationFilter::CosignedOnly.label()}</option>
                                                </select>
                                            </div>

                                            <div style="display:flex;align-items:center;justify-content:space-between;gap:.5rem;">
                                                <label style="font-size:.78rem;color:var(--text-secondary);display:flex;align-items:center;gap:.45rem;cursor:pointer;">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=move || include_unknown.get()
                                                        on:change=move |ev| set_include_unknown.set(event_target_checked(&ev))
                                                    />
                                                    "Afficher INCONNU"
                                                </label>
                                                <button
                                                    class="btn"
                                                    style="padding:.35rem .6rem;font-size:.75rem;"
                                                    on:click=move |_| {
                                                        set_search.set(String::new());
                                                        set_relation_filter.set(RelationFilter::All);
                                                        set_limit_rows.set(100);
                                                    }
                                                >
                                                    "Reset"
                                                </button>
                                            </div>

                                            <div>
                                                <label style="display:block;font-size:.76rem;color:var(--text-muted);margin-bottom:.35rem;">"Lignes affichées"</label>
                                                <select
                                                    class="input"
                                                    style="width:100%;"
                                                    on:change=move |ev| {
                                                        let v = event_target_value(&ev);
                                                        let parsed = v.parse::<usize>().unwrap_or(100);
                                                        set_limit_rows.set(parsed.max(20));
                                                    }
                                                    prop:value=move || limit_rows.get().to_string()
                                                >
                                                    <option value="50">"50"</option>
                                                    <option value="100">"100"</option>
                                                    <option value="250">"250"</option>
                                                    <option value="1000">"1000"</option>
                                                </select>
                                            </div>
                                        </div>

                                        <div style="padding:.85rem;border:1px solid var(--bg-border);border-radius:10px;background:var(--bg-secondary);">
                                            <h3 style="margin:0 0 .45rem 0;font-size:.85rem;">"Groupes disponibles"</h3>
                                            <div style="display:flex;flex-wrap:wrap;gap:.35rem;max-height:220px;overflow:auto;">
                                                {index_for_chips.groups.iter().filter(|g| include_unknown.get() || g.group_id != "INCONNU").map(|g| {
                                                    let gid = g.group_id.clone();
                                                    let active = selected_group_id.get().as_deref() == Some(g.group_id.as_str());
                                                    let style = if active {
                                                        "border:1px solid var(--accent);background:rgba(34,197,94,.08);color:var(--text-primary);"
                                                    } else {
                                                        "border:1px solid var(--bg-border);background:var(--bg-secondary);color:var(--text-secondary);"
                                                    };
                                                    view! {
                                                        <button
                                                            on:click=move |_| set_selected_group_id.set(Some(gid.clone()))
                                                            style={format!("padding:.3rem .55rem;border-radius:999px;font-size:.72rem;cursor:pointer;{}", style)}
                                                            title={format!("{} — {} entrées", g.group_label, g.ppl_count)}
                                                        >
                                                            {format!("{} ({})", g.group_label, g.ppl_count)}
                                                        </button>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    </div>

                                    <div style="display:flex;flex-direction:column;gap:.85rem;">
                                        {move || match shard_res.get() {
                                            None => view! {
                                                <div style="padding:1rem;border:1px solid var(--bg-border);border-radius:10px;background:var(--bg-secondary);color:var(--text-muted);">
                                                    "Chargement du shard groupe..."
                                                </div>
                                            }.into_view(),
                                            Some(Err(e)) => view! {
                                                <div style="padding:1rem;border:1px solid var(--danger);border-radius:10px;background:rgba(239,68,68,.08);color:var(--danger);">
                                                    {format!("Erreur de chargement du shard groupe : {}", e)}
                                                </div>
                                            }.into_view(),
                                            Some(Ok(None)) => view! {
                                                <div style="padding:1rem;border:1px solid var(--bg-border);border-radius:10px;background:var(--bg-secondary);color:var(--text-muted);">
                                                    "Sélectionnez un groupe pour afficher les PPL associées."
                                                </div>
                                            }.into_view(),
                                            Some(Ok(Some(shard))) => {
                                                let filtered = filtered_items.get();
                                                let shown = displayed_items.get();
                                                let total_filtered = filtered.len();
                                                let hidden = total_filtered.saturating_sub(shown.len());
                                                let authors = filtered.iter().filter(|i| i.has_author).count();
                                                let cosigned_only = total_filtered.saturating_sub(authors);

                                                view! {
                                                    <>
                                                        <section style="padding:.9rem;border:1px solid var(--bg-border);border-radius:10px;background:var(--bg-secondary);display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                                                            <div>
                                                                <h2 style="margin:0 0 .25rem 0;font-size:1rem;font-weight:700;">
                                                                    {format!("{} · Propositions de loi", shard.group_label)}
                                                                </h2>
                                                                <p style="margin:0;color:var(--text-muted);font-size:.76rem;line-height:1.4;">
                                                                    {format!("Shard {} · {} entrées brutes dans ce groupe.", shard.group_id, shard.total_entries)}
                                                                    {" "}
                                                                    {match &shard.generated_at {
                                                                        s if !s.is_empty() => format!("Généré le {}.", s),
                                                                        _ => String::new(),
                                                                    }}
                                                                </p>
                                                            </div>
                                                            <div style="display:grid;grid-template-columns:repeat(3,minmax(110px,1fr));gap:.4rem;min-width:min(100%,420px);">
                                                                <MiniStatBox label="Après filtres" value=total_filtered.to_string() />
                                                                <MiniStatBox label="Auteur présent" value=authors.to_string() />
                                                                <MiniStatBox label="Cosignature seule" value=cosigned_only.to_string() />
                                                            </div>
                                                        </section>

                                                        {if total_filtered == 0 {
                                                            view! {
                                                                <div style="padding:1rem;border:1px solid var(--bg-border);border-radius:10px;background:var(--bg-secondary);color:var(--text-muted);">
                                                                    "Aucune entrée ne correspond aux filtres actuels."
                                                                </div>
                                                            }.into_view()
                                                        } else {
                                                            view! {
                                                                <div style="padding:.55rem .75rem;border:1px solid var(--bg-border);border-radius:10px;background:var(--bg-secondary);display:flex;justify-content:space-between;gap:.75rem;align-items:center;flex-wrap:wrap;">
                                                                    <span style="font-size:.77rem;color:var(--text-muted);">
                                                                        {format!("{} ligne(s) affichée(s) sur {} après filtres.", shown.len(), total_filtered)}
                                                                    </span>
                                                                    {if hidden > 0 {
                                                                        view! { <span style="font-size:.74rem;color:var(--text-muted);">{format!("{} lignes masquées par la limite", hidden)}</span> }.into_view()
                                                                    } else {
                                                                        view!{<></>}.into_view()
                                                                    }}
                                                                </div>
                                                            }.into_view()
                                                        }}

                                                        <div style="border:1px solid var(--bg-border);border-radius:10px;overflow:hidden;background:var(--bg-secondary);">
                                                            <table class="data-table" style="table-layout:fixed;">
                                                                <colgroup>
                                                                    <col style="width:92px;" />
                                                                    <col style="width:84px;" />
                                                                    <col />
                                                                    <col style="width:160px;" />
                                                                    <col style="width:140px;" />
                                                                </colgroup>
                                                                <thead>
                                                                    <tr>
                                                                        <th>"Lég."</th>
                                                                        <th>"Type"</th>
                                                                        <th>"Proposition de loi"</th>
                                                                        <th>"Signataires du groupe"</th>
                                                                        <th>"Aperçu"</th>
                                                                    </tr>
                                                                </thead>
                                                                <tbody>
                                                                    {shown.into_iter().map(render_row).collect_view()}
                                                                </tbody>
                                                            </table>
                                                        </div>
                                                    </>
                                                }.into_view()
                                            }
                                        }}
                                    </div>
                                </section>
                            </>
                        }.into_view()
                    }
                }
            }}
        </div>
    }
}

fn render_row(item: GroupPplItemSummary) -> impl IntoView {
    let relation_badge = if item.has_author {
        view! {
            <span style="display:inline-flex;align-items:center;padding:.14rem .45rem;border-radius:999px;font-size:.68rem;font-weight:600;background:rgba(34,197,94,.12);color:#22c55e;border:1px solid rgba(34,197,94,.25);">
                "Auteur"
            </span>
        }
    } else {
        view! {
            <span style="display:inline-flex;align-items:center;padding:.14rem .45rem;border-radius:999px;font-size:.68rem;font-weight:600;background:rgba(59,130,246,.12);color:#60a5fa;border:1px solid rgba(59,130,246,.25);">
                "Cosig."
            </span>
        }
    };

    let title_main = item.title.clone();
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

    let counts_label = if item.author_count > 0 && item.cosigner_count > 0 {
        format!("{} auteur(s), {} cosig.", item.author_count, item.cosigner_count)
    } else if item.author_count > 0 {
        format!("{} auteur(s)", item.author_count)
    } else {
        format!("{} cosig.", item.cosigner_count)
    };

    let preview_signers = item.signers_preview.clone();
    let preview_names = item.signer_names_preview.clone();
    let preview_view = render_signers_preview_cell(preview_signers, preview_names);

    let source_url_trimmed = item
        .source_url
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())
        .map(|u| u.to_string());

    let title_view = if let Some(url) = source_url_trimmed.clone() {
        let title_url = url.clone();
        let link_url = url;
        view! {
            <div style="display:flex;align-items:flex-start;gap:.35rem;flex-wrap:wrap;">
                <a href=title_url target="_blank" rel="noopener noreferrer" style="color:var(--text-primary);text-decoration:none;font-weight:600;line-height:1.35;">
                    {title_main}
                </a>
                <a
                    href=link_url
                    target="_blank"
                    rel="noopener noreferrer"
                    style="font-size:.70rem;color:var(--text-muted);text-decoration:none;border:1px solid var(--bg-border);border-radius:999px;padding:.05rem .35rem;line-height:1.2;"
                    title="Ouvrir la page Assemblée nationale"
                >
                    "AN ↗"
                </a>
            </div>
        }.into_view()
    } else {
        view! { <span style="font-weight:600;line-height:1.35;">{title_main}</span> }.into_view()
    };

    view! {
        <tr>
            <td>
                {item.legislature.map(|l| format!("L{}", l)).unwrap_or_else(|| "—".to_string())}
            </td>
            <td>{relation_badge}</td>
            <td>
                <div style="display:flex;flex-direction:column;gap:.2rem;">
                    {title_view}
                    {if !title_meta.is_empty() {
                        view! { <span style="font-size:.72rem;color:var(--text-muted);">{title_meta}</span> }.into_view()
                    } else { view!{<></>}.into_view() }}
                    <span style="font-size:.70rem;color:var(--text-muted);">{item.ppl_id}</span>
                </div>
            </td>
            <td>
                <div style="display:flex;flex-direction:column;gap:.2rem;">
                    <span style="font-weight:600;">{item.total_signers_from_group.to_string()}</span>
                    <span style="font-size:.72rem;color:var(--text-muted);">{counts_label}</span>
                </div>
            </td>
            <td style="font-size:.74rem;color:var(--text-secondary);line-height:1.35;">
                {preview_view}
            </td>
        </tr>
    }
}

fn render_signers_preview_cell(signers: Vec<SignerPreviewEntry>, fallback_names: Vec<String>) -> View {
    if !signers.is_empty() {
        return view! {
            <div style="display:flex;flex-wrap:wrap;gap:.25rem;align-items:center;">
                {signers.into_iter().map(|s| {
                    let label = s.deputy_name.trim().to_string();
                    let dep_id = s.deputy_id.as_deref().map(str::trim).filter(|id| !id.is_empty()).map(|id| id.to_string());
                    if let Some(dep_id) = dep_id {
                        let href = app_href(&format!("/depute/{dep_id}"));
                        view! {
                            <A href=href attr:style="color:var(--accent);text-decoration:none;">{label}</A>
                        }.into_view()
                    } else {
                        view! {
                            <span>{label}</span>
                        }.into_view()
                    }
                }).collect_view()}
            </div>
        }
        .into_view();
    }

    if fallback_names.is_empty() {
        return view! { <span style="color:var(--text-muted);">"—"</span> }.into_view();
    }

    view! {
        <span>{fallback_names.join(", ")}</span>
    }
    .into_view()
}

#[component]
fn MiniStatCard(label: &'static str, value: String, sub: String) -> impl IntoView {
    view! {
        <div style="padding:.75rem .8rem;border:1px solid var(--bg-border);border-radius:10px;background:var(--bg-secondary);display:flex;flex-direction:column;gap:.15rem;">
            <span style="font-size:.72rem;color:var(--text-muted);">{label}</span>
            <strong style="font-size:1.05rem;line-height:1.1;">{value}</strong>
            <span style="font-size:.70rem;color:var(--text-muted);line-height:1.35;">{sub}</span>
        </div>
    }
}

#[component]
fn MiniStatBox(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div style="padding:.45rem .55rem;border:1px solid var(--bg-border);border-radius:8px;background:rgba(255,255,255,.01);display:flex;flex-direction:column;gap:.1rem;">
            <span style="font-size:.68rem;color:var(--text-muted);">{label}</span>
            <strong style="font-size:.92rem;line-height:1.1;">{value}</strong>
        </div>
    }
}
