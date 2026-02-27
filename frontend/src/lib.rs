mod api;
mod components;
mod models;
mod pages;
mod store;
mod utils;

use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;

use components::layout::Layout;
use pages::{
    amendements::AmendementsPage,
    comparer::ComparerPage, depute::DeputePage, exporter::ExportPage, home::HomePage,
    methodologie::MethodePage, positions_groupes::PositionsGroupesPage, reseau::ReseauPage,
    stats_globales::StatsGlobalesPage,
};
use store::provide_store;

use std::sync::OnceLock;

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
        <Router base=crate::app_path!("")>
            <Layout>
                <Routes>
                    <Route path=crate::app_path!("/home") view=HomePage />
                    <Route path=crate::app_path!("/") view=HomePage />
                    <Route path=crate::app_path!("/depute/:id") view=DeputePage />
                    <Route path=crate::app_path!("/comparer") view=ComparerPage />
                    <Route path=crate::app_path!("/exporter") view=ExportPage />
                    <Route path=crate::app_path!("/stats-globales") view=StatsGlobalesPage />
                    <Route path=crate::app_path!("/amendements") view=AmendementsPage />
                    <Route path=crate::app_path!("/reseau") view=ReseauPage />
                    <Route path=crate::app_path!("/positions-groupes") view=PositionsGroupesPage />
                    <Route path=crate::app_path!("/methodologie") view=MethodePage />
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
            <A href=crate::app_path!("/home") class="btn">"Retour à l'accueil"</A>
        </div>
    }
}
