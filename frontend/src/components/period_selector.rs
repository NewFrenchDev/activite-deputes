use leptos::*;
use crate::models::Period;
use crate::store::use_store;

#[component]
pub fn PeriodSelector(
    period: ReadSignal<Period>,
    set_period: WriteSignal<Period>,
) -> impl IntoView {
    let store = use_store();

    view! {
        <div style="display:flex;gap:0.25rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:7px;padding:3px;">
            <PeriodBtn label="30 j" p=Period::P30 current=period set=set_period store=store.clone() />
            <PeriodBtn label="180 j" p=Period::P180 current=period set=set_period store=store.clone() />
            <PeriodBtn label="Législature" p=Period::LEG current=period set=set_period store=store.clone() />
        </div>
    }
}

#[component]
fn PeriodBtn(
    label: &'static str,
    p: Period,
    current: ReadSignal<Period>,
    set: WriteSignal<Period>,
    store: crate::store::AppStore,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| {
                // Si l'utilisateur clique sur "Législature" et que les données ne sont pas encore chargées, les charger
                if p == Period::LEG && !store.is_leg_loaded() {
                    store.load_leg();
                }
                set.set(p)
            }
            style=move || {
                let active = current.get() == p;
                if active {
                    "padding:0.3rem 0.75rem;border-radius:5px;border:none;cursor:pointer;font-size:0.78rem;font-weight:600;background:var(--accent);color:#000;transition:all 0.15s;"
                } else {
                    "padding:0.3rem 0.75rem;border-radius:5px;border:none;cursor:pointer;font-size:0.78rem;font-weight:500;background:transparent;color:var(--text-secondary);transition:all 0.15s;"
                }
            }
        >
            {label}
        </button>
    }
}
