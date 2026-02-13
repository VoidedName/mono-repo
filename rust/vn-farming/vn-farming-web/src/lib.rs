use js_sys::Promise;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use vn_farming_logic::PlatformHooks;
use vn_scene::GenericScene;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(module = "/src/helpers.js")]
extern "C" {
    pub fn load_file_js(path: &str) -> Promise;
    pub fn exit();
    pub fn loaded();
}

pub async fn load_file(path: String) -> Result<Vec<u8>, String> {
    let promise = load_file_js(&path);

    let file = match JsFuture::from(promise).await {
        Ok(file) => file,
        Err(e) => {
            return Err(String::from(
                e.as_string()
                    .unwrap_or_else(|| "Failed to load file".to_string()),
            ));
        }
    };

    let file_buffer = file.dyn_into::<js_sys::ArrayBuffer>().unwrap();
    let file_bytes = js_sys::Uint8Array::new(&file_buffer);
    Ok(file_bytes.to_vec())
}

pub struct WebPlatformHooks;
impl PlatformHooks for WebPlatformHooks {
    fn execute_async(&self, f: impl Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(f);
    }

    fn load_asset(
        &self,
        path: impl AsRef<Path>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>, String>>>> {
        Box::pin(load_file(
            PathBuf::from("assets")
                .join(path)
                .to_string_lossy()
                .to_string(),
        ))
    }

    fn logic_was_initialized(&self) {
        loaded()
    }

    fn exit(&self) {
        exit();
        std::process::exit(0);
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("Could not initialize logger");
    vn_farming_logic::init::<GenericScene, WebPlatformHooks>(WebPlatformHooks)
        .expect("Failed to initialize logic");
    log::info!("Farming web started");
}
