use std::rc::Rc;
use std::cell::RefCell;
use web_sys::AudioContext;

/// Returns the shared AudioContext, creating it on first call.
/// Must be called from a user gesture handler (browser requirement).
pub fn ensure_audio_context(ctx: &Rc<RefCell<Option<AudioContext>>>) -> AudioContext {
    let mut borrow = ctx.borrow_mut();
    if borrow.is_none() {
        *borrow = Some(AudioContext::new().expect("AudioContext::new failed"));
    }
    borrow.as_ref().unwrap().clone()
}
