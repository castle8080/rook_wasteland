use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::poem_repository::{fetch_index, fetch_poem, pick_random, PoemDetail};
use crate::ui::recording_controls::RecordingControls;

/// Full poem reader view: loads and displays a random poem.
/// Supports `?poem_id=<id>` query param to load a specific poem (e.g., from recording detail).
#[component]
pub fn ReaderView() -> impl IntoView {
    let query = use_query_map();
    // Incrementing this signal re-triggers the resource (New Poem / Try again).
    let refresh = RwSignal::new(0u32);
    // Track the current poem id to exclude it from the next random pick.
    let current_poem_id: RwSignal<Option<String>> = RwSignal::new(None);

    // Two-step fetch: load index then pick and fetch a random poem.
    let poem_resource: LocalResource<Result<PoemDetail, String>> =
        LocalResource::new(move || {
            let requested_id = query.read().get("poem_id").map(|s| s.to_string());
            let exclude = current_poem_id.get();
            let _refresh = refresh.get(); // tracked — triggers re-run on New Poem
            async move {
                let index = fetch_index().await?;

                // If a specific poem_id was requested via query param, load it.
                if let Some((path, entry_id)) = requested_id.and_then(|id| {
                    index.poems.iter().find(|e| e.id == id)
                        .map(|e| (e.path.clone(), e.id.clone()))
                }) {
                    let poem = fetch_poem(&path).await?;
                    current_poem_id.set(Some(entry_id));
                    return Ok(poem);
                }

                let entry = pick_random(&index, exclude.as_deref())
                    .ok_or_else(|| "Poem index is empty.".to_string())?;
                let path = entry.path.clone();
                let id = entry.id.clone();
                let poem = fetch_poem(&path).await?;
                // Update the current id after a successful fetch.
                current_poem_id.set(Some(id));
                Ok(poem)
            }
        });

    let on_new_poem = move |_| {
        refresh.update(|n| *n += 1);
    };

    view! {
        <main class="content-column" lang="en">
            {move || {
                match poem_resource.get() {
                    None => view! {
                        <p class="state-message">"Loading poem…"</p>
                    }.into_any(),
                    Some(Err(e)) => view! {
                        <div class="state-message">
                            <p>{format!("Unable to load poems. Check your connection. ({e})")}</p>
                            <button class="btn btn-secondary" on:click=move |_| refresh.update(|n| *n += 1)>
                                "Try again"
                            </button>
                        </div>
                    }.into_any(),
                    Some(Ok(poem)) => {
                        let poem_id = poem.id.clone();
                        let poem_title = poem.title.clone();
                        let poem_author = poem.author.clone();
                        view! {
                            <article>
                                <h1 class="poem-title">{poem.title.clone()}</h1>
                                <p class="poem-meta">
                                    {poem.author.clone()}
                                    {poem.date.as_ref().map(|d| format!(" · {d}"))}
                                </p>
                                <pre class="poem-body">{poem.content.clone()}</pre>
                            </article>
                            <RecordingControls
                                poem_id=poem_id
                                poem_title=poem_title
                                poem_author=poem_author
                            />
                            <div style="margin-top: 1.5rem;">
                                <button class="btn btn-secondary" on:click=on_new_poem>
                                    "New Poem"
                                </button>
                            </div>
                        }.into_any()
                    }
                }
            }}
        </main>
    }
}
