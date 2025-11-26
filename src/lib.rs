pub mod core;
pub mod platform;
pub mod render;
pub mod ecs;
pub mod resources;
pub mod physics;
pub mod editor;
pub mod scripting;
pub mod services;
pub mod audio;
pub mod bindings;
pub mod xr;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    core::Engine::run();
}

// Hint: On WASM, complex UI can be rendered using DOM overlay for better accessibility and text handling.
// Use web-sys to manipulate DOM elements layered on top of the canvas.

