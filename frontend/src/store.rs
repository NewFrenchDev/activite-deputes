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
    pub stats_leg: RwSignal<Option<Resource<(), Result<Vec<DeputeStats>, String>>>>,
}

impl AppStore {
    pub fn new() -> Self {
        Self {
            status:     create_resource(|| (), |_| fetch_status()),
            stats_p30:  create_resource(|| (), |_| fetch_stats(Period::P30)),
            stats_p180: create_resource(|| (), |_| fetch_stats(Period::P180)),
            stats_leg:  RwSignal::new(None), 
        }
    }

    /// Charger LEG à la demande (appelé quand utilisateur clique sur "Législature")
    pub fn load_leg(&self) {
        // Si déjà chargé, ne rien faire
        if self.stats_leg.get().is_some() {
            return;
        }
        
        // Créer la ressource et la stocker
        let resource = create_resource(|| (), |_| fetch_stats(Period::LEG));
        self.stats_leg.set(Some(resource));
    }

    /// Pour LEG : crée une resource "vide" si pas chargé
    pub fn get_resource_for(&self, period: Period) -> Resource<(), Result<Vec<DeputeStats>, String>> {
        match period {
            Period::P30  => self.stats_p30,
            Period::P180 => self.stats_p180,
            Period::LEG  => {
                // Si LEG pas chargé, retourner une ressource vide
                if let Some(resource) = self.stats_leg.get() {
                    resource
                } else {
                    // Créer une ressource qui retourne une liste vide
                    create_resource(|| (), |_| async {
                        Ok(vec![])  // Liste vide si pas chargé
                    })
                }
            }
        }
    }

    pub fn stats_for(&self, period: Period) -> Resource<(), Result<Vec<DeputeStats>, String>> {
        self.get_resource_for(period)
    }

    /// Chercher un député
    pub fn find_depute(&self, period: Period, id: &str) -> Option<DeputeStats> {
        // Récupérer la Resource
        let resource = self.stats_for(period);
        
        // .get() sur Resource retourne Option<Result<Vec<DeputeStats>, String>>
        resource
            .get()
            .and_then(|result| result.ok())  // Extraire le Vec du Result
            .and_then(|data| data.into_iter().find(|d| d.deputy_id == id))
    }

    /// Vérifier si LEG est chargé (utile pour l'UI)
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
