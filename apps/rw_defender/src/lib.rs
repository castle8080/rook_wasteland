mod entities;
mod game;
mod graphics;
mod renderer;
mod systems;
mod utils;

use game::{Game, CANVAS_H, CANVAS_W};
use graphics::{background_image_for_wave, BackgroundTier, StarField};
use renderer::Renderer;
use systems::InputState;

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, KeyboardEvent};

// Use a thread-local to hold shared game state accessible from JS callbacks.
thread_local! {
    static GAME: RefCell<Option<Game>> = const { RefCell::new(None) };
    static INPUT: RefCell<InputState> = RefCell::new(InputState::new());
    static RENDERER: RefCell<Option<Renderer>> = const { RefCell::new(None) };
    static STARFIELD: RefCell<Option<StarField>> = const { RefCell::new(None) };
    static LAST_TIME: RefCell<f64> = const { RefCell::new(0.0) };
    /// Wave number for which the current background image is loaded (MAX = not yet set).
    static BG_WAVE: RefCell<u32> = const { RefCell::new(u32::MAX) };
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    let window = window().expect("no global window");
    let document = window.document().expect("no document");

    // Get game canvas
    let canvas = document
        .get_element_by_id("game-canvas")
        .expect("no #game-canvas element")
        .dyn_into::<HtmlCanvasElement>()
        .expect("#game-canvas is not a canvas");

    let ctx = canvas
        .get_context("2d")
        .expect("get_context failed")
        .expect("no 2d context")
        .dyn_into::<CanvasRenderingContext2d>()
        .expect("context not CanvasRenderingContext2d");

    let renderer = Renderer::new(ctx, CANVAS_W, CANVAS_H);

    // Get background canvas for the starfield
    let bg_canvas = document
        .get_element_by_id("background-canvas")
        .expect("no #background-canvas element")
        .dyn_into::<HtmlCanvasElement>()
        .expect("#background-canvas is not a canvas");

    let starfield = StarField::new(&bg_canvas);

    GAME.with(|g| *g.borrow_mut() = Some(Game::new()));
    RENDERER.with(|r| *r.borrow_mut() = Some(renderer));
    STARFIELD.with(|s| *s.borrow_mut() = Some(starfield));

    // Set up keyboard event listeners
    setup_input_listeners(&window);

    // Start game loop
    start_game_loop();
}

fn setup_input_listeners(window: &web_sys::Window) {
    // keydown
    let keydown = Closure::wrap(Box::new(|event: KeyboardEvent| {
        // Prevent default for game keys to stop page scrolling
        match event.key().as_str() {
            " " | "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" => {
                event.prevent_default();
            }
            _ => {}
        }
        INPUT.with(|i| i.borrow_mut().handle_keydown(&event.key()));
    }) as Box<dyn FnMut(KeyboardEvent)>);

    window
        .add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())
        .expect("keydown listener failed");
    keydown.forget();

    // keyup
    let keyup = Closure::wrap(Box::new(|event: KeyboardEvent| {
        INPUT.with(|i| i.borrow_mut().handle_keyup(&event.key()));
    }) as Box<dyn FnMut(KeyboardEvent)>);

    window
        .add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())
        .expect("keyup listener failed");
    keyup.forget();
}

type RafClosure = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;

/// Update the `#space-bg` img src when the wave changes, cycling through NASA/ESA images.
fn maybe_update_background(wave: u32) {
    let needs_update = BG_WAVE.with(|bw| {
        let current = *bw.borrow();
        if current != wave {
            *bw.borrow_mut() = wave;
            true
        } else {
            false
        }
    });
    if !needs_update {
        return;
    }
    let filename = background_image_for_wave(wave);
    let src = format!("/backgrounds/{filename}");
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Some(el) = doc.get_element_by_id("space-bg") {
                if let Ok(img) = el.dyn_into::<HtmlImageElement>() {
                    img.set_src(&src);
                }
            }
        }
    }
}



fn start_game_loop() {
    let f: RafClosure = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
        // Compute delta time
        let dt = LAST_TIME.with(|lt| {
            let prev = *lt.borrow();
            let delta = if prev == 0.0 { 0.016 } else { (timestamp - prev) / 1000.0 };
            *lt.borrow_mut() = timestamp;
            delta
        });

        // Update game
        INPUT.with(|input| {
            GAME.with(|game| {
                if let Some(ref mut g) = *game.borrow_mut() {
                    g.update(&mut input.borrow_mut(), dt);
                }
            });
        });

        // Update and render starfield (background layer); update NASA image on wave change
        GAME.with(|game| {
            STARFIELD.with(|sf| {
                if let (Some(ref g), Some(ref mut s)) = (&*game.borrow(), &mut *sf.borrow_mut()) {
                    let tier = BackgroundTier::for_wave(g.wave);
                    if s.tier != tier {
                        s.set_tier(tier);
                    }
                    maybe_update_background(g.wave);
                    s.update(dt);
                    s.render();
                }
            });
        });

        // Render game
        GAME.with(|game| {
            RENDERER.with(|renderer| {
                if let (Some(ref g), Some(ref r)) = (&*game.borrow(), &*renderer.borrow()) {
                    g.render(r);
                }
            });
        });

        // Schedule next frame
        window()
            .unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .expect("rAF failed");
    }) as Box<dyn FnMut(f64)>));

    window()
        .unwrap()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .expect("rAF failed");
}
