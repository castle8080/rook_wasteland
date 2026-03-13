pub mod deck;
pub mod mixer;

// `pub use` re-exports below are flagged as unused_imports by rustc when compiling
// the rlib target for `cargo test`, because the test binary doesn't import them
// through this path.  They are genuinely used by other modules via `crate::state::*`
// in the cdylib (WASM) target.  The allow is correct — do not remove it.
#[allow(unused_imports)]
pub use deck::{DeckState, ZoomLevel};
#[allow(unused_imports)]
pub use mixer::{DeckId, MixerState};

// Leptos context is keyed by TypeId.  Two values of the same type would overwrite
// each other, so each deck state needs its own unique wrapper type.
/// Context wrapper for Deck A's `DeckState`.  Use with `provide_context` /
/// `use_context` to disambiguate from Deck B.
#[derive(Clone)]
pub struct DeckAContext(pub DeckState);
/// Context wrapper for Deck B's `DeckState`.
#[derive(Clone)]
pub struct DeckBContext(pub DeckState);
