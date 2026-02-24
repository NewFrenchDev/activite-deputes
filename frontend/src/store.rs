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
    pub stats_leg:  Resource<(), Result<Vec<DeputeStats>, String>>,
}

impl AppStore {
    pub fn new() -> Self {
        Self {
            status:     create_resource(|| (), |_| fetch_status()),
            stats_p30:  create_resource(|| (), |_| fetch_stats(Period::P30)),
            stats_p180: create_resource(|| (), |_| fetch_stats(Period::P180)),
            stats_leg:  create_resource(|| (), |_| fetch_stats(Period::Leg)),
        }
    }

    pub fn stats_for(&self, period: Period) -> Resource<(), Result<Vec<DeputeStats>, String>> {
        match period {
            Period::P30  => self.stats_p30,
            Period::P180 => self.stats_p180,
            Period::Leg  => self.stats_leg,
        }
    }

    /// Cherche un député par ID dans le dataset chargé pour une période donnée.
    pub fn find_depute(&self, period: Period, id: &str) -> Option<DeputeStats> {
        self.stats_for(period)
            .get()
            .and_then(|r| r.ok())
            .and_then(|v| v.into_iter().find(|d| d.deputy_id == id))
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
