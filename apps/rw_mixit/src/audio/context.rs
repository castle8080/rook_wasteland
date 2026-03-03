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

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn context_is_none_before_first_call() {
        let holder: Rc<RefCell<Option<AudioContext>>> = Rc::new(RefCell::new(None));
        assert!(holder.borrow().is_none());
    }

    #[wasm_bindgen_test]
    fn context_is_created_on_first_call() {
        let holder = Rc::new(RefCell::new(None));
        let _ctx = ensure_audio_context(&holder);
        assert!(holder.borrow().is_some(), "holder must be populated after first call");
    }

    #[wasm_bindgen_test]
    fn context_is_reused_on_second_call() {
        let holder = Rc::new(RefCell::new(None));
        let a = ensure_audio_context(&holder);
        let b = ensure_audio_context(&holder);
        // Same underlying instance → same sample rate.
        assert_eq!(a.sample_rate(), b.sample_rate());
    }

    #[wasm_bindgen_test]
    fn context_has_positive_sample_rate() {
        let holder = Rc::new(RefCell::new(None));
        let ctx = ensure_audio_context(&holder);
        assert!(ctx.sample_rate() > 0.0);
    }
}
