use leptos::*;

#[component]
pub fn Tooltip(text: &'static str, children: Children) -> impl IntoView {
    view! {
        <span class="tooltip-trigger" style="position:relative;">
            {children()}
            <span class="tooltip-content" role="tooltip">{text}</span>
        </span>
    }
}

#[component]
pub fn InfoIcon(text: &'static str) -> impl IntoView {
    view! {
        <Tooltip text=text>
            <button style="background:none;border:none;cursor:help;color:var(--text-muted);font-size:0.75rem;padding:0 0.2rem;display:inline-flex;align-items:center;" aria-label="Information">
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"/>
                    <path d="M12 16v-4M12 8h.01"/>
                </svg>
            </button>
        </Tooltip>
    }
}
