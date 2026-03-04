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
