use env_logger::Env;
use std::io::Read;
use std::pin::Pin;
use vn_farming_logic::logic::{PlatformHooks, FileLoadingError};

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
    fn load_file(
        &self,
        path: String,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>, FileLoadingError>>>> {
        Box::pin(load_file(format!("assets/{}", path)))
    }

    fn exit(&self) {
        std::process::exit(0);
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

    vn_farming_logic::init(Box::new(NativePlatformHooks)).expect("Failed to initialize!");
}
