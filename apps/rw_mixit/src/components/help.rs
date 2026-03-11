use leptos::prelude::*;

/// Quick-start guide page — a short numbered walkthrough in the app's
/// cartoon hip-hop voice.  Purely presentational; no signals or context.
#[component]
pub fn HelpView() -> impl IntoView {
    view! {
        <main class="help-view">
            <section class="help-card">
                <h1 class="help-title">"🎛️ how to get hyped"</h1>
                <p class="help-tagline">
                    "Four moves. That's all it takes to go from zero to DJ hero."
                </p>

                <ol class="help-steps">
                    <li class="help-step">
                        <strong class="help-step-title">"📂 Load your tracks"</strong>
                        " — Click the folder icon in the top-right corner of each deck "
                        "and pick an audio file from your computer. Do it for Deck A and "
                        "Deck B. No upload, no login — it stays on your machine."
                    </li>
                    <li class="help-step">
                        <strong class="help-step-title">"▶️ Drop the beat"</strong>
                        " — Hit PLAY on both decks. Use the pitch fader to nudge the "
                        "tempo until the two tracks feel like they're vibing together. "
                        "Tap BPM if you need a hand."
                    </li>
                    <li class="help-step">
                        <strong class="help-step-title">"🎚️ Dial the levels"</strong>
                        " — The A and B faders in the center mixer control each deck's "
                        "volume independently. Get them balanced before you blend."
                    </li>
                    <li class="help-step">
                        <strong class="help-step-title">"🎵 Work the crossfader"</strong>
                        " — Slide it left for Deck A, right for Deck B, center for the "
                        "blend. Ease it across slowly for a smooth transition — or slam "
                        "it hard for a cut. That's a mix, baby. 🔥"
                    </li>
                </ol>

                <p class="help-footer">
                    "Want more? Explore the EQ knobs, FX panel, loop controls, and hot "
                    "cues on each deck. Hit "
                    <a href="#/settings" class="help-link">"Settings"</a>
                    " to tweak the reverb and crossfader curve."
                </p>
            </section>
        </main>
    }
}
