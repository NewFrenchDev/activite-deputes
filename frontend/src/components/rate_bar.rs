use leptos::*;
use crate::utils::{fmt_pct, participation_class};

#[component]
pub fn RateBar(rate: f64) -> impl IntoView {
    let pct_str = fmt_pct(rate);
    let fill_pct = (rate * 100.0).clamp(0.0, 100.0);
    let cls = participation_class(rate);
    view! {
        <span style="display:inline-flex;align-items:center;gap:0.4rem;">
            <span class="rate-bar-bg">
                <span class="rate-bar-fill" style=format!("width:{}%;", fill_pct)></span>
            </span>
            <span class=cls style="font-size:0.8rem;font-variant-numeric:tabular-nums;font-weight:500;">{pct_str}</span>
        </span>
    }
}
