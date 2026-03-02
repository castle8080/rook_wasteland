use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::ui::{
    components::TopBar, reader::ReaderView, recording_detail::RecordingDetailView,
    recordings_list::RecordingsListView,
};

/// Top-level App component with router and top bar.
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <TopBar />
            <Routes fallback=|| view! { <p class="text-secondary">"Page not found."</p> }>
                <Route path=path!("/") view=ReaderView />
                <Route path=path!("/readings") view=RecordingsListView />
                <Route path=path!("/readings/:recording_id") view=RecordingDetailView />
            </Routes>
        </Router>
    }
}
