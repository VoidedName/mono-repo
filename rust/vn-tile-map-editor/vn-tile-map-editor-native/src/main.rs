use env_logger::Env;
use rfd::{AsyncFileDialog, FileDialog};
use std::future::Future;
use std::io::Read;
use std::path::PathBuf;
use std::pin::Pin;
use vn_tile_map_editor_logic::logic::{FileLoadingError, PlatformHooks};

pub async fn load_file(path: String) -> anyhow::Result<Vec<u8>, FileLoadingError> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| FileLoadingError::GeneralError(format!("Failed to open file: {}", e)))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| FileLoadingError::GeneralError(format!("Failed to read file: {}", e)))?;
    Ok(buffer)
}

struct NativePlatformHooks;
impl PlatformHooks for NativePlatformHooks {
    fn load_asset(
        &self,
        path: String,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>, FileLoadingError>>>> {
        Box::pin(load_file(format!("assets/{}", path)))
    }

    fn load_file(
        &self,
        path: String,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>, FileLoadingError>>>> {
        Box::pin(load_file(format!("{}", path)))
    }

    fn exit(&self) {
        std::process::exit(0);
    }

    fn pick_file(&self, extensions: &[&str]) -> Option<String> {
        pollster::block_on(async {
            AsyncFileDialog::new()
                .add_filter("filter", extensions)
                .pick_file().await
                .map(|path| path.path().to_str().map(String::from))
                .flatten()
        })
    }
}

fn main() {
    let log_level = std::env::var("MY_LOG_LEVEL")
        .unwrap_or_else(|_| "Debug, wgpu_hal=WARN, wgpu_core=WARN, naga=WARN".to_string());
    let log_style = std::env::var("MY_LOG_STYLE").unwrap_or_else(|_| "always".to_string());

    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", &log_level)
        .write_style_or("MY_LOG_STYLE", &log_style);
    env_logger::init_from_env(env);

    log::info!(
        "Logging initialized. MY_LOG_LEVEL: {}, MY_LOG_STYLE: {}",
        log_level,
        log_style
    );

    vn_tile_map_editor_logic::init(Box::new(NativePlatformHooks)).expect("Failed to initialize!");
}
