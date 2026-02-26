use leptos::*;
use wasm_bindgen::JsCast;

use crate::store::use_store;
use crate::api::{stats_to_csv, base_url};
use crate::models::Period;

#[component]
pub fn ExportPage() -> impl IntoView {
    let store = use_store();

    let exports = vec![
        (Period::P30,  "Activité — 30 derniers jours glissants"),
        (Period::P180, "Activité — 180 derniers jours glissants"),
        (Period::LEG,  "Activité — Depuis début de législature / mandat"),
    ];

    view! {
        <div class="reveal">
            <h1 style="font-size:1.4rem;font-weight:700;margin:0 0 0.25rem 0;">"Exporter les données"</h1>
            <p style="color:var(--text-muted);font-size:0.82rem;margin-bottom:1.5rem;">
                "Fichiers CSV et JSON générés depuis les sources open data officielles. "
                "Les exports CSV sont aussi générables directement depuis le navigateur (compatible mobile)."
            </p>

            {move || store.status.get().and_then(|r| r.ok()).map(|s| view! {
                <div style="margin-bottom:1.5rem;padding:0.75rem 1rem;background:var(--accent-dim);border:1px solid var(--accent-border);border-radius:6px;font-size:0.8rem;">
                    "Dernière mise à jour : "
                    <strong style="color:var(--text-primary);">{s.last_update_readable}</strong>
                    " · "
                    <span style="color:var(--text-muted);">{format!("{} députés", s.counts.deputes)}</span>
                </div>
            })}

            <section style="margin-bottom:2rem;">
                <h2 style="font-size:0.9rem;font-weight:600;margin:0 0 1rem 0;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);">
                    "Exports CSV & JSON"
                </h2>
                <div style="display:flex;flex-direction:column;gap:0.75rem;">
                    {exports.iter().map(|(period, desc)| {
                        let p = *period;
                        let resource = store.stats_for(p);
                        let json_url = format!("{}/{}", base_url(), p.json_file());

                        view! {
                            <div style="display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:0.75rem;padding:1rem 1.25rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:8px;">
                                <div>
                                    <p style="font-weight:500;margin:0 0 0.2rem 0;font-size:0.9rem;">{*desc}</p>
                                    <p style="color:var(--text-muted);font-size:0.75rem;margin:0;font-family:monospace;">
                                        {p.csv_file()}
                                    </p>
                                </div>
                                <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                                    // CSV généré côté client — fonctionne sur iOS Safari
                                    <button
                                        class="btn"
                                        on:click=move |_| {
                                            if let Some(Ok(ref data)) = resource.get() {
                                                let csv = stats_to_csv(data);
                                                trigger_download(&csv, &format!("{}.csv", p.csv_label()), "text/csv;charset=utf-8;");
                                            }
                                        }
                                    >
                                        <DownloadIcon />
                                        "CSV (navigateur)"
                                    </button>
                                    // Lien JSON statique
                                    <a href=json_url class="btn" target="_blank" rel="noopener">
                                        <DownloadIcon />
                                        "JSON"
                                    </a>
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </section>

            // Schéma colonnes
            <section style="margin-bottom:2rem;">
                <h2 style="font-size:0.9rem;font-weight:600;margin:0 0 1rem 0;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);">
                    "Schéma des colonnes CSV"
                </h2>
                <div style="background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:8px;overflow:hidden;overflow-x:auto;">
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>"Colonne"</th>
                                <th>"Type"</th>
                                <th>"Description"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {CSV_SCHEMA.iter().map(|(col, typ, desc)| view! {
                                <tr>
                                    <td style="font-family:monospace;font-size:0.78rem;color:var(--accent);white-space:nowrap;">{*col}</td>
                                    <td style="font-size:0.75rem;color:var(--text-muted);white-space:nowrap;">{*typ}</td>
                                    <td style="font-size:0.78rem;">{*desc}</td>
                                </tr>
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
            </section>

            // Sources
            <section>
                <h2 style="font-size:0.9rem;font-weight:600;margin:0 0 1rem 0;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);">
                    "Sources téléchargées"
                </h2>
                {move || store.status.get().and_then(|r| r.ok()).map(|s| view! {
                    <div style="display:flex;flex-direction:column;gap:0.5rem;">
                        {s.sources.into_iter().map(|src| view! {
                            <div style="padding:0.6rem 1rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:6px;font-size:0.78rem;display:flex;gap:1rem;align-items:center;flex-wrap:wrap;">
                                <span style="color:var(--accent);font-family:monospace;min-width:100px;">{src.key}</span>
                                <span style="color:var(--text-muted);">
                                    {format!("{} Ko", src.size_bytes / 1024)}
                                </span>
                                {src.last_modified.map(|lm| view! {
                                    <span style="color:var(--text-muted);">"modifié : "{lm}</span>
                                })}
                                {src.etag.map(|et| view! {
                                    <span style="color:var(--text-muted);font-family:monospace;font-size:0.7rem;">
                                        "ETag: "{et}
                                    </span>
                                })}
                            </div>
                        }).collect_view()}
                    </div>
                })}
            </section>
        </div>
    }
}

/// Déclenche un téléchargement de fichier texte compatible iOS Safari
/// En créant un Blob + URL objet puis en cliquant sur un lien temporaire
fn trigger_download(content: &str, filename: &str, mime: &str) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };

    // Créer un Blob avec le contenu CSV
    let array = js_sys::Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(content));

    let blob_opts = web_sys::BlobPropertyBag::new();
    blob_opts.set_type(mime);

    let blob = match web_sys::Blob::new_with_str_sequence_and_options(&array, &blob_opts) {
        Ok(b) => b,
        Err(_) => return,
    };

    let url = match web_sys::Url::create_object_url_with_blob(&blob) {
        Ok(u) => u,
        Err(_) => return,
    };

    // Créer un <a> temporaire et cliquer dessus
    if let Ok(a) = document.create_element("a") {
        let a: web_sys::HtmlAnchorElement = a.unchecked_into();
        a.set_href(&url);
        a.set_download(filename);
        a.style().set_property("display", "none").ok();
        if let Some(body) = document.body() {
            body.append_child(&a).ok();
            a.click();
            body.remove_child(&a).ok();
        }
    }

    // Libérer l'URL objet après un court délai
    let url_clone = url.clone();
    let closure = wasm_bindgen::closure::Closure::once_into_js(move || {
        web_sys::Url::revoke_object_url(&url_clone).ok();
    });
    window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        1000,
    ).ok();
}

#[component]
fn DownloadIcon() -> impl IntoView {
    view! {
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
        </svg>
    }
}

// Ajout d'une méthode csv_label sur Period
trait CsvLabel { fn csv_label(&self) -> &str; }
impl CsvLabel for Period {
    fn csv_label(&self) -> &str {
        match self {
            Period::P30  => "deputes_activity_P30",
            Period::P180 => "deputes_activity_P180",
            Period::LEG  => "deputes_activity_LEG",
        }
    }
}

const CSV_SCHEMA: &[(&str, &str, &str)] = &[
    ("deputy_id",          "string",        "Identifiant unique acteur (données AN)"),
    ("nom",                "string",        "Nom de famille"),
    ("prenom",             "string",        "Prénom"),
    ("groupe_abrev",       "string",        "Abréviation du groupe parlementaire"),
    ("groupe_nom",         "string",        "Libellé complet du groupe parlementaire"),
    ("parti_rattachement", "string|null",   "Parti politique déclaré (absent si non disponible)"),
    ("dept",               "string",        "Nom du département"),
    ("circo",              "string",        "Numéro de circonscription"),
    ("period_start",       "YYYY-MM-DD",   "Début période effective (max période / début mandat)"),
    ("period_end",         "YYYY-MM-DD",   "Fin période effective"),
    ("scrutins_eligibles", "integer",       "Scrutins publics dans la période avec mandat actif"),
    ("votes_exprimes",     "integer",       "Positions Pour + Contre + Abstention enregistrées"),
    ("non_votant",         "integer",       "Positions NON_VOTANT enregistrées"),
    ("absent",             "integer",       "Scrutins éligibles sans position enregistrée"),
    ("participation_rate", "float [0-1]",  "votes_exprimes / scrutins_eligibles"),
    ("pour_count",         "integer",       "Votes POUR"),
    ("contre_count",       "integer",       "Votes CONTRE"),
    ("abst_count",         "integer",       "Abstentions"),
    ("amd_authored",       "integer",       "Amendements déposés comme auteur principal"),
    ("amd_adopted",        "integer",       "Amendements déposés adoptés"),
    ("amd_adoption_rate",  "float|null",   "amd_adopted / amd_authored (null si authored=0)"),
    ("amd_cosigned",       "integer",       "Amendements co-signés (auteur secondaire)"),
    ("interventions_count","integer",       "Interventions en séance (0 en V1)"),
    ("interventions_chars","integer",       "Caractères interventions (0 en V1)"),
    ("top_dossier_id",     "string|null",   "Dossier avec score d'activité le plus élevé"),
    ("top_dossier_titre",  "string|null",   "Titre du dossier principal"),
    ("top_dossier_score",  "integer|null",  "Score = 1×votes + 2×amendements"),
];
