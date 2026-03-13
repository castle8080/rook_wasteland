use leptos::prelude::*;

use crate::routing::Route;

/// Top navigation bar: logo (links to main view) and nav links for Settings
/// and About views.
#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="rw-header">
            <a
                class="rw-logo"
                href="#/"
                on:click=move |e| {
                    e.prevent_default();
                    let _ = web_sys::window()
                        .expect("window unavailable")
                        .location()
                        .set_hash(Route::Main.to_hash());
                }
            >
                "rw_mixit"
            </a>
            <nav class="rw-nav">
                <a
                    href="#/settings"
                    on:click=move |e| {
                        e.prevent_default();
                        let _ = web_sys::window()
                            .expect("window unavailable")
                            .location()
                            .set_hash(Route::Settings.to_hash());
                    }
                >
                    "[Settings]"
                </a>
                <a
                    href="#/help"
                    on:click=move |e| {
                        e.prevent_default();
                        let _ = web_sys::window()
                            .expect("window unavailable")
                            .location()
                            .set_hash(Route::Help.to_hash());
                    }
                >
                    "[Help]"
                </a>
                <a
                    href="#/about"
                    on:click=move |e| {
                        e.prevent_default();
                        let _ = web_sys::window()
                            .expect("window unavailable")
                            .location()
                            .set_hash(Route::About.to_hash());
                    }
                >
                    "[About]"
                </a>
            </nav>
        </header>
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    mod wasm {
        #![allow(clippy::let_unit_value, clippy::unwrap_used)]
        use leptos::prelude::*;
        use leptos::mount::mount_to_body;
        use wasm_bindgen::JsCast;
        use wasm_bindgen_test::wasm_bindgen_test;

        use crate::components::header::Header;

        fn win() -> web_sys::Window {
            web_sys::window().unwrap()
        }

        fn click_selector(selector: &str) {
            win()
                .document()
                .unwrap()
                .query_selector(selector)
                .unwrap()
                .expect("element not found")
                .unchecked_ref::<web_sys::HtmlElement>()
                .click();
        }

        fn current_hash() -> String {
            win().location().hash().unwrap_or_default()
        }

        #[wasm_bindgen_test]
        fn logo_click_sets_main_hash() {
            let _handle = mount_to_body(|| view! { <Header/> });
            click_selector("a.rw-logo");
            assert_eq!(current_hash(), Route::Main.to_hash());
        }

        #[wasm_bindgen_test]
        fn settings_link_sets_settings_hash() {
            let _handle = mount_to_body(|| view! { <Header/> });
            click_selector(".rw-nav a[href='#/settings']");
            assert_eq!(current_hash(), Route::Settings.to_hash());
        }

        #[wasm_bindgen_test]
        fn about_link_sets_about_hash() {
            let _handle = mount_to_body(|| view! { <Header/> });
            click_selector(".rw-nav a[href='#/about']");
            assert_eq!(current_hash(), Route::About.to_hash());
        }

        #[wasm_bindgen_test]
        fn help_link_sets_help_hash() {
            let _handle = mount_to_body(|| view! { <Header/> });
            click_selector(".rw-nav a[href='#/help']");
            assert_eq!(current_hash(), Route::Help.to_hash());
        }

        use crate::routing::Route;
    }
}

