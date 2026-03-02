use leptos::mount::mount_to_body;

mod app;
mod audio_capture;
mod poem_repository;
mod recording_store;
mod ui;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(app::App);
}
