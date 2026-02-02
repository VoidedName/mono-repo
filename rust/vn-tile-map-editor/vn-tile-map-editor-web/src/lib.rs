use std::path::PathBuf;
use std::pin::Pin;
use wasm_bindgen::prelude::*;

use js_sys::Promise;
use vn_tile_map_editor_logic::logic::{File, FileDescriptor, FileLoadingError, PlatformHooks};
use vn_wgpu_window::WgpuScene;
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

fn divide_path(path: &str) -> (String, String, Option<String>) {
    let path = PathBuf::from(path);
    let extension = path.extension().map(|e| e.to_string_lossy().to_string());
    let name = path
        .file_stem()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();
    (
        path.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        name,
        extension,
    )
}

#[derive(Clone, Debug)]
struct WebPlatformHooks;
impl PlatformHooks for WebPlatformHooks {
    fn execute_async(&self, f: impl Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(f);
    }

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

    fn pick_file(&self, extensions: &[&str]) -> Pin<Box<dyn Future<Output = Option<File>>>> {
        let extensions = extensions
            .into_iter()
            .cloned()
            .map(String::from)
            .collect::<Vec<_>>();
        Box::pin(async move {
            match rfd::AsyncFileDialog::new()
                .add_filter("", &extensions)
                .pick_file()
                .await
            {
                Some(file) => {
                    let (parent, name, extension) = divide_path(&file.file_name());

                    Some(File {
                        descriptor: FileDescriptor {
                            path: parent,
                            extension,
                            name,
                        },
                        bytes: file.read().await,
                    })
                }
                None => None,
            }
        })
    }

    fn save_file(
        &self,
        suggested_name: &str,
        extensions: &[&str],
        bytes: &[u8],
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>>>> {
        let extensions = extensions
            .into_iter()
            .cloned()
            .map(String::from)
            .collect::<Vec<_>>();
        let data = bytes.to_vec();
        let name = suggested_name.to_string();

        Box::pin(async move {
            match rfd::AsyncFileDialog::new()
                .add_filter("", &extensions)
                .set_file_name(&name)
                .set_title("Download Tilemap")
                .save_file()
                .await
            {
                Some(file) => {
                    file.write(&data).await?;
                    Ok(())
                }
                None => Err(anyhow::anyhow!("No file selected")),
            }
        })
    }
}

#[wasm_bindgen]
pub fn main_web() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize console_log");
    log::info!("Logging initialized with level: {:?}", log::Level::Info);

    vn_tile_map_editor_logic::init::<WgpuScene, WebPlatformHooks>(WebPlatformHooks)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(())
}
