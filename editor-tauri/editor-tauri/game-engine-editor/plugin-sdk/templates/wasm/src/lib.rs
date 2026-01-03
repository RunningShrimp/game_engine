//! {{plugin-name}} (WASM)

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn name() -> String {
    "{{plugin-name}}".to_string()
}

#[wasm_bindgen]
pub fn version() -> String {
    "0.1.0".to_string()
}

#[wasm_bindgen]
pub fn description() -> String {
    "{{description}}".to_string()
}

#[wasm_bindgen]
pub fn author() -> String {
    "{{author}}".to_string()
}

#[wasm_bindgen]
pub fn on_load() -> i32 {
    // Plugin initialization
    // Return 0 for success, non-zero for error
    0
}

#[wasm_bindgen]
pub fn on_update(delta_time: f32) {
    // Called every frame
    // TODO: Implement update logic
    let _ = delta_time;
}

#[wasm_bindgen]
pub fn on_unload() -> i32 {
    // Cleanup
    // Return 0 for success, non-zero for error
    0
}
