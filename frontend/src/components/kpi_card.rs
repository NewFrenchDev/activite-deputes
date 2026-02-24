use leptos::*;

#[component]
pub fn KpiCard(
    label: &'static str,
    value: String,
    #[prop(optional)] sub: Option<String>,
    #[prop(optional)] color: Option<&'static str>,
) -> impl IntoView {
    let color = color.unwrap_or("var(--text-primary)");
    view! {
        <div class="kpi-card reveal">
            <p style="font-size:0.7rem;font-weight:600;letter-spacing:0.06em;text-transform:uppercase;color:var(--text-muted);margin:0 0 0.5rem 0;">
                {label}
            </p>
            <p style=format!("font-size:1.8rem;font-weight:700;margin:0 0 0.25rem 0;color:{color};font-variant-numeric:tabular-nums;") class="count-up">
                {value}
            </p>
            {sub.map(|s| view! {
                <p style="font-size:0.75rem;color:var(--text-muted);margin:0;">{s}</p>
            })}
        </div>
    }
}
