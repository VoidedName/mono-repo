use std::pin::Pin;
use wasm_bindgen::prelude::*;

use js_sys::Promise;
use vn_tile_map_editor_logic::logic::{PlatformHooks, FileLoadingError, File};
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(module = "/src/helpers.js")]
extern "C" {
    pub fn load_file_js(path: &str) -> Promise;
    pub fn exit();
    pub fn loaded();
}

pub async fn load_file(path: String) -> Result<Vec<u8>, FileLoadingError> {
    let promise = load_file_js(&path);

    let file = match JsFuture::from(promise).await {
        Ok(file) => file,
        Err(e) => {
            return Err(FileLoadingError::GeneralError(String::from(
                e.as_string()
                    .unwrap_or_else(|| "Failed to load file".to_string()),
            )));
        }
    };

    let file_buffer = file.dyn_into::<js_sys::ArrayBuffer>().unwrap();
    let file_bytes = js_sys::Uint8Array::new(&file_buffer);
    Ok(file_bytes.to_vec())
}

struct WebPlatformHooks;
impl PlatformHooks for WebPlatformHooks {
    fn has_initialized() {
        loaded();
    }

    fn load_asset(
        &self,
        path: String,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>, FileLoadingError>>>> {
        Box::pin(load_file(format!("assets/{}", path)))
    }

    fn exit(&self) {
        exit();
        std::process::exit(0);
    }

    fn pick_file(&self, extensions: &[&str]) -> Option<File> {
        todo!()
    }

    fn save_file(&self, extensions: &[&str], bytes: &[u8]) -> anyhow::Result<()> {
        todo!()
    }
}

#[wasm_bindgen]
pub fn main_web() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize console_log");
    log::info!("Logging initialized with level: {:?}", log::Level::Info);

    vn_tile_map_editor_logic::init(WebPlatformHooks).map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(())
}
