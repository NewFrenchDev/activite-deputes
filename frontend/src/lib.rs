mod api;
mod components;
mod models;
mod pages;
mod store;
mod utils;

use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;

use crate::utils::{app_base_path, app_href};
use components::layout::Layout;
use pages::{
    comparer::ComparerPage, depute::DeputePage, exporter::ExportPage, home::HomePage,
    methodologie::MethodePage, positions_groupes::PositionsGroupesPage, reseau::ReseauPage,
    stats_globales::StatsGlobalesPage,
};
use store::provide_store;

use std::sync::OnceLock;

static BASE_PATH: OnceLock<&'static str> = OnceLock::new();

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    // Store global initialisé une seule fois à la racine
    provide_store();

    // Le router Leptos attend un base sans slash final (sauf "/").
    // On garde une valeur stable pour éviter toute divergence au runtime.
    let base: &'static str = *BASE_PATH.get_or_init(|| Box::leak(app_base_path().into_boxed_str()));

    view! {
        <Router base=base>
            <Layout>
                <Routes>
                    <Route path="" view=HomePage />
                    <Route path="depute/:id" view=DeputePage />
                    <Route path="comparer" view=ComparerPage />
                    <Route path="exporter" view=ExportPage />
                    <Route path="stats-globales" view=StatsGlobalesPage />
                    <Route path="reseau" view=ReseauPage />
                    <Route path="positions-groupes" view=PositionsGroupesPage />
                    <Route path="methodologie" view=MethodePage />
                    <Route path="/*any" view=|| view! { <NotFound /> } />
                </Routes>
            </Layout>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div style="display:flex;flex-direction:column;align-items:center;justify-content:center;min-height:60vh;gap:1rem;">
            <p style="font-size:3rem;color:var(--text-muted)">404</p>
            <p style="color:var(--text-secondary)">"Page non trouvée"</p>
            <A href=app_href("/") class="btn">"Retour à l'accueil"</A>
        </div>
    }
}
