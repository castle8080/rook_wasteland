use leptos::prelude::*;

/// Static About page: version number, stack description, and credits.
#[component]
pub fn AboutView() -> impl IntoView {
    view! {
        <main class="about-view">
            <section class="about-card">
                <h1 class="about-title">"rw_mixit"</h1>
                <p class="about-version">"v0.1.0"</p>
                <p class="about-tagline">
                    "Browser-based DJ mixer — dual-deck turntable with BPM detection, "
                    "live FX, and loop controls. No server. No install."
                </p>

                <h2>"Stack"</h2>
                <ul class="about-stack">
                    <li>"Rust / WASM (wasm32-unknown-unknown)"</li>
                    <li>"Leptos 0.8 (CSR)"</li>
                    <li>"Web Audio API — AudioContext, ConvolverNode, DelayNode, AnalyserNode"</li>
                    <li>"Canvas 2D — platter animation, waveform scrolling"</li>
                    <li>"Trunk (build / hot-reload)"</li>
                </ul>

                <h2>"Part of Rook Wasteland"</h2>
                <p>
                    "A collection of independent client-side WASM apps deployed as a "
                    "unified static file hosting solution."
                </p>
            </section>
        </main>
    }
}
