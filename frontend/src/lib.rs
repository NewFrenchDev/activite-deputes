mod models;
mod api;
mod store;
mod pages;
mod components;
mod utils;

use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;

use store::provide_store;
use pages::{
    home::HomePage,
    depute::DeputePage,
    comparer::ComparerPage,
    exporter::ExportPage,
    methodologie::MethodePage,
    stats_globales::StatsGlobalesPage,
    reseau::ReseauPage,
    positions_groupes::PositionsGroupesPage,
};
use components::layout::Layout;
use crate::utils::{app_base_path, app_href};

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    // Store global initialisé une seule fois à la racine
    provide_store();

    view! {
        <Router base="/">
            <Layout>
                <Routes>
                    <Route path="/" view=HomePage />
                    <Route path="/depute/:id" view=DeputePage />
                    <Route path="/comparer" view=ComparerPage />
                    <Route path="/exporter" view=ExportPage />
                    <Route path="/stats-globales" view=StatsGlobalesPage />
                    <Route path="/reseau" view=ReseauPage />
                    <Route path="/positions-groupes" view=PositionsGroupesPage />
                    <Route path="/methodologie" view=MethodePage />
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
