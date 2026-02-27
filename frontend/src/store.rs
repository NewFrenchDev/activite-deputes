use leptos::*;
use crate::api::{fetch_stats, fetch_status};
use crate::models::{DeputeStats, Period, Status};

/// Store global partagé via context Leptos.
/// Les données sont chargées une fois et réutilisées sur toutes les pages.
#[derive(Clone)]
pub struct AppStore {
    pub status: Resource<(), Result<Status, String>>,
    pub stats_p30:  Resource<(), Result<Vec<DeputeStats>, String>>,
    pub stats_p180: Resource<(), Result<Vec<DeputeStats>, String>>,
    /// LEG: Option Ressource chargée à la demande
    pub stats_leg:  RwSignal<Option<Resource<(), Result<Vec<DeputeStats>, String>>>>,
    /// Placeholder unique réutilisable tant que LEG n'est pas chargé
    pub stats_leg_placeholder: Resource<(), Result<Vec<DeputeStats>, String>>,
}

impl AppStore {
    pub fn new() -> Self {
        // Ressources P30/P180 chargées normalement
        let stats_p30 = create_resource(|| (), |_| fetch_stats(Period::P30));
        let stats_p180 = create_resource(|| (), |_| fetch_stats(Period::P180));

        // LEG préchargé au démarrage pour minimiser le temps de chargement
        let stats_leg_resource = create_resource(|| (), |_| fetch_stats(Period::LEG));

        // Placeholder: ressource statique qui retourne une liste vide (créée une seule fois)
        // On l'utilise pour éviter de créer une Resource différente à chaque appel
        let stats_leg_placeholder: Resource<(), Result<Vec<DeputeStats>, String>> =
            create_resource(|| (), |_| async { Ok(Vec::new()) });

        Self {
            status:     create_resource(|| (), |_| fetch_status()),
            stats_p30,
            stats_p180,
            stats_leg:  RwSignal::new(Some(stats_leg_resource)),
            stats_leg_placeholder,
        }
    }

    /// Charger LEG à la demande (appelé quand utilisateur clique sur "Législature")
    pub fn load_leg(&self) {
        if self.stats_leg.get().is_some() {
            return;
        }
        // créer la ressource LEG réelle qui télécharge le fichier
        let resource = create_resource(|| (), |_| fetch_stats(Period::LEG));
        self.stats_leg.set(Some(resource));
    }

    /// Retourne TOUJOURS une Resource: pour LEG, renvoie le placeholder si pas encore chargé
    pub fn get_resource_for(&self, period: Period) -> Resource<(), Result<Vec<DeputeStats>, String>> {
        match period {
            Period::P30  => self.stats_p30,
            Period::P180 => self.stats_p180,
            Period::LEG  => {
                if let Some(resource) = self.stats_leg.get() {
                    resource
                } else {
                    // retourner le placeholder statique (même instance à chaque appel)
                    self.stats_leg_placeholder
                }
            }
        }
    }

    /// Compatibilité ancienne API
    pub fn stats_for(&self, period: Period) -> Resource<(), Result<Vec<DeputeStats>, String>> {
        self.get_resource_for(period)
    }

    /// Cherche un député par ID
    pub fn find_depute(&self, period: Period, id: &str) -> Option<DeputeStats> {
        self.stats_for(period)
            .get()
            .and_then(|r| r.ok())
            .and_then(|v| v.into_iter().find(|d| d.deputy_id == id))
    }

    pub fn is_leg_loaded(&self) -> bool {
        self.stats_leg.get().is_some()
    }
}

pub fn provide_store() -> AppStore {
    let store = AppStore::new();
    provide_context(store.clone());
    store
}

pub fn use_store() -> AppStore {
    use_context::<AppStore>().expect("AppStore must be provided at root")
}