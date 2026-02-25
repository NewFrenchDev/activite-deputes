use leptos::*;
use leptos_router::*;

use crate::api::{fetch_status, inferred_github_repo_urls};
use crate::utils::app_href;

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let status_res = create_resource(|| (), |_| fetch_status());
    let repo_links = inferred_github_repo_urls();
    let repo_url = repo_links.as_ref().map(|(r, _)| r.clone());
    let issue_url = repo_links.as_ref().map(|(_, i)| i.clone());
    let (theme, set_theme) = create_signal(String::from("dark"));

    let toggle_theme = move |_| {
        let next = if theme.get() == "dark" { "light" } else { "dark" };
        set_theme.set(next.to_string());
        if let Some(html) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            let _ = html.set_attribute("class", next);
        }
    };

    view! {
        <div style="min-height:100vh;display:flex;flex-direction:column;">
            <header style="background:var(--bg-secondary);border-bottom:1px solid var(--bg-border);position:sticky;top:0;z-index:50;">
                <div style="background:linear-gradient(90deg, rgba(245,158,11,.16), rgba(245,158,11,.06));border-bottom:1px solid rgba(245,158,11,.22);">
                    <div style="max-width:1400px;margin:0 auto;padding:0.35rem 1.5rem;display:flex;align-items:center;justify-content:space-between;gap:0.75rem;flex-wrap:wrap;">
                        <div style="display:flex;align-items:center;gap:0.5rem;flex-wrap:wrap;">
                            <span style="font-size:0.65rem;font-weight:700;letter-spacing:.04em;padding:0.15rem 0.45rem;border-radius:999px;background:rgba(245,158,11,.2);border:1px solid rgba(245,158,11,.35);color:var(--warning);">"BÊTA PUBLIQUE"</span>
                            <span style="font-size:0.76rem;color:var(--text-secondary);">"Le site est fonctionnel mais en amélioration continue (UX, liens AN, couverture données)."</span>
                        </div>
                        <div style="display:flex;align-items:center;gap:.7rem;flex-wrap:wrap;">
                            <A href=app_href("/methodologie") attr:style="font-size:0.75rem;color:var(--accent);text-decoration:none;">"Méthode & limites"</A>
                            {match issue_url.clone() {
                                Some(url) => view! {
                                    <a href=url target="_blank" rel="noopener noreferrer" style="font-size:0.75rem;color:var(--accent);text-decoration:none;">"Signaler un problème ↗"</a>
                                }.into_view(),
                                None => view! {
                                    <A href=app_href("/methodologie#retours") attr:style="font-size:0.75rem;color:var(--accent);text-decoration:none;">"Feedback"</A>
                                }.into_view(),
                            }}
                        </div>
                    </div>
                </div>
                <div style="max-width:1400px;margin:0 auto;padding:0 1.5rem;display:flex;align-items:center;justify-content:space-between;height:56px;">
                    <div style="display:flex;align-items:center;gap:2rem;">
                        <A href=app_href("/") attr:style="display:flex;align-items:center;gap:0.6rem;text-decoration:none;">
                            <span style="width:28px;height:28px;background:var(--accent);border-radius:6px;display:flex;align-items:center;justify-content:center;">
                                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2.5">
                                    <rect x="3" y="3" width="18" height="18" rx="2"/>
                                    <path d="M3 9h18M9 21V9"/>
                                </svg>
                            </span>
                            <span style="font-weight:700;font-size:0.95rem;color:var(--text-primary)">Activité Députés</span>
                            <span style="font-size:0.62rem;padding:0.12rem 0.35rem;border-radius:999px;border:1px solid rgba(245,158,11,.35);color:var(--warning);background:rgba(245,158,11,.08);font-weight:600;">"BETA"</span>
                            <span style="font-size:0.65rem;color:var(--text-muted);font-weight:400;margin-top:2px;">"17e législature"</span>
                        </A>
                        <nav style="display:flex;gap:0;padding-left:1rem;border-left:1px solid var(--bg-border);" aria-label="Navigation principale">
                            <NavLink path="/" label="Accueil" />
                            <NavLink path="/comparer" label="Comparer" />
                            <NavLink path="/exporter" label="Exporter" />
                            <NavLink path="/stats-globales" label="Stats globales" />
                            <NavLink path="/reseau" label="Réseau" />
                            <NavLink path="/positions-groupes" label="Positions groupes" />
                            <NavLink path="/methodologie" label="Méthode & Sources" />
                        </nav>
                    </div>
                    <div style="display:flex;align-items:center;gap:0.75rem;">
                        <button
                            on:click=toggle_theme
                            style="background:none;border:1px solid var(--bg-border);border-radius:6px;padding:0.35rem 0.6rem;cursor:pointer;color:var(--text-secondary);font-size:0.8rem;"
                            title="Basculer thème clair/sombre"
                        >
                            {move || if theme.get() == "dark" { "☀ Clair" } else { "⚫ Sombre" }}
                        </button>
                    </div>
                </div>
            </header>

            <main style="flex:1;max-width:1400px;margin:0 auto;padding:1.5rem;width:100%;">
                {children()}
            </main>

            <footer style="border-top:1px solid var(--bg-border);padding:1.1rem 1.5rem;text-align:center;background:var(--bg-secondary);">
                <div style="max-width:1400px;margin:0 auto;display:flex;flex-direction:column;gap:0.45rem;align-items:center;">
                    <p style="font-size:0.75rem;color:var(--text-muted);margin:0;">
                        "Données : "
                        <a href="https://data.assemblee-nationale.fr" target="_blank" rel="noopener" style="color:var(--accent);">
                            "data.assemblee-nationale.fr"
                        </a>
                        " — Open data Assemblée nationale — Licence Ouverte v2.0"
                        " · Ce site ne contient pas d'opinion ni de commentaire éditorial. "
                        <A href=app_href("/methodologie") attr:style="color:var(--accent);">"Méthode & Sources"</A>
                    </p>
                    <div style="font-size:0.74rem;color:var(--text-muted);display:flex;align-items:center;gap:.45rem;flex-wrap:wrap;justify-content:center;">
                        {match repo_url.clone() {
                            Some(url) => view! { <a href=url target="_blank" rel="noopener noreferrer" style="color:var(--accent);text-decoration:none;">"Code source GitHub ↗"</a> }.into_view(),
                            None => view! { <span>"Code source : lien GitHub à ajouter dans le README / footer"</span> }.into_view(),
                        }}
                        <span style="opacity:.55;">"·"</span>
                        {match issue_url.clone() {
                            Some(url) => view! { <a href=url target="_blank" rel="noopener noreferrer" style="color:var(--accent);text-decoration:none;">"Feedback / Issues ↗"</a> }.into_view(),
                            None => view! { <A href=app_href("/methodologie#retours") attr:style="color:var(--accent);text-decoration:none;">"Comment signaler un problème"</A> }.into_view(),
                        }}
                    </div>
                    <div style="font-size:0.74rem;color:var(--text-muted);">
                        {move || match status_res.get() {
                            None => view! { <span>"Mise à jour des données : vérification..."</span> }.into_view(),
                            Some(Err(_)) => view! { <span>"Mise à jour des données : indisponible (status.json non chargé)"</span> }.into_view(),
                            Some(Ok(s)) => view! {
                                <span>
                                    "Dernière mise à jour des données : "
                                    <strong style="color:var(--text-secondary);">{s.last_update_readable}</strong>
                                </span>
                            }.into_view(),
                        }}
                    </div>
                </div>
            </footer>
        </div>
    }
}

#[component]
fn NavLink(path: &'static str, label: &'static str) -> impl IntoView {
    let href = app_href(path);
    view! {
        <A
            href=href
            attr:style="padding:0 0.85rem;height:56px;display:flex;align-items:center;font-size:0.82rem;color:var(--text-secondary);text-decoration:none;border-bottom:2px solid transparent;transition:all 0.15s;"
            active_class="nav-active"
            exact=true
        >
            {label}
        </A>
    }
}
