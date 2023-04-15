use wasm_bindgen::prelude::*;

pub mod error;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
fn init() -> Result<(), JsValue> {
    Ok(())
}
