use std::io::Read;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use vn_farming_logic::PlatformHooks;
use vn_scene::GenericScene;

pub async fn load_file(path: PathBuf) -> anyhow::Result<Vec<u8>, String> {
    let mut file =
        std::fs::File::open(path.clone()).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    Ok(buffer)
}

struct NativePlatformHooks;
impl PlatformHooks for NativePlatformHooks {
    fn execute_async(&self, f: impl Future<Output = ()> + 'static) {
        pollster::block_on(f);
    }

    fn load_asset(
        &self,
        path: impl AsRef<Path>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>, String>>>> {
        Box::pin(load_file(PathBuf::from("assets").join(path)))
    }

    fn logic_was_initialized(&self) {}

    fn exit(&self) {
        std::process::exit(0);
    }
}

fn main() {
    env_logger::init();
    vn_farming_logic::init::<GenericScene, NativePlatformHooks>(NativePlatformHooks)
        .expect("Failed to initialize!");
    log::info!("Farming native started");
}
