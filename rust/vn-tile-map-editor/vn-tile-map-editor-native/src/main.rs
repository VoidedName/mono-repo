use env_logger::Env;
use rfd::AsyncFileDialog;
use std::future::Future;
use std::io::Read;
use std::path::PathBuf;
use std::pin::Pin;
use vn_tile_map_editor_logic::logic::{File, FileDescriptor, FileLoadingError, PlatformHooks};

pub async fn load_file(file_to_load: FileDescriptor) -> anyhow::Result<File, FileLoadingError> {
    let mut path = PathBuf::from(&file_to_load.path).join(&file_to_load.name);
    if let Some(extension) = &file_to_load.extension {
        path = path.with_added_extension(extension);
    }

    let mut file = std::fs::File::open(path.clone())
        .map_err(|e| FileLoadingError::GeneralError(format!("Failed to open file: {}", e)))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| FileLoadingError::GeneralError(format!("Failed to read file: {}", e)))?;

    let (parent, name, extension) = divide_path(&path.to_string_lossy());

    Ok(File {
        descriptor: FileDescriptor {
            path: parent,
            name,
            extension,
        },
        bytes: buffer,
    })
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

struct NativePlatformHooks;
impl PlatformHooks for NativePlatformHooks {
    fn block_on<T>(future: impl Future<Output = T>) -> T {
        pollster::block_on(future)
    }

    fn load_asset(
        &self,
        path: String,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>, FileLoadingError>>>> {
        Box::pin(async move {
            load_file(FileDescriptor {
                path: "assets".to_string(),
                name: path,
                extension: None,
            })
            .await
            .map(|f| f.bytes)
        })
    }

    fn load_file(
        &self,
        file: &FileDescriptor,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<File, FileLoadingError>>>> {
        Box::pin(load_file(file.clone()))
    }

    fn exit(&self) {
        std::process::exit(0);
    }

    fn pick_file(&self, extensions: &[&str]) -> Option<File> {
        pollster::block_on(async {
            let path = AsyncFileDialog::new()
                .add_filter("filter", extensions)
                .pick_file()
                .await
                .map(|path| {
                    let path = path.path().to_string_lossy().to_string();
                    divide_path(&path)
                });

            match path {
                Some((parent, name, extension)) => self
                    .load_file(&FileDescriptor {
                        path: parent,
                        name,
                        extension,
                    })
                    .await
                    .ok(),
                None => None,
            }
        })
    }

    fn pick_folder(&self) -> Option<String> {
        pollster::block_on(async {
            AsyncFileDialog::new()
                .set_can_create_directories(true)
                .pick_folder()
                .await
                .map(|path| path.path().to_string_lossy().to_string())
        })
    }

    fn save_file(&self, file: File) -> anyhow::Result<()> {
        let path = std::path::Path::new(&file.descriptor.path);
        let mut path = path.join(file.descriptor.name);
        if let Some(extension) = &file.descriptor.extension {
            path = path.with_added_extension(extension);
        }

        std::fs::write(path, file.bytes)?;
        Ok(())
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

    vn_tile_map_editor_logic::init(NativePlatformHooks).expect("Failed to initialize!");
}
