use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_web() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize console_log");
    log::info!("Logging initialized with level: {:?}", log::Level::Info);

    vn_vttrpg::init().map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(())
}
