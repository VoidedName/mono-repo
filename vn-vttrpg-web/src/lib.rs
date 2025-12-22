use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_web() -> Result<(), JsValue> {
    vn_vttrpg::init().map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(())
}