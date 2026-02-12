#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Include all game logic modules
mod game;

// Export game types and systems for WASM
pub use game::*;

/// WASM entry point - auto-runs when module loads
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    game::run_game();
}
