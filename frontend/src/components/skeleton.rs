use leptos::*;

#[component]
pub fn SkeletonRow(cols: usize) -> impl IntoView {
    view! {
        <tr>
            {(0..cols).map(|_| view! {
                <td style="padding:0.55rem 0.75rem;">
                    <div class="skeleton" style="height:14px;width:80%;"></div>
                </td>
            }).collect_view()}
        </tr>
    }
}

#[component]
pub fn SkeletonTable(rows: usize, cols: usize) -> impl IntoView {
    view! {
        <tbody>
            {(0..rows).map(|_| view! { <SkeletonRow cols=cols /> }).collect_view()}
        </tbody>
    }
}

#[component]
pub fn SkeletonKpi() -> impl IntoView {
    view! {
        <div class="kpi-card">
            <div class="skeleton" style="height:12px;width:60%;margin-bottom:0.75rem;"></div>
            <div class="skeleton" style="height:28px;width:40%;margin-bottom:0.5rem;"></div>
            <div class="skeleton" style="height:10px;width:70%;"></div>
        </div>
    }
}
