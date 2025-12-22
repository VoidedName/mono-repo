#[cfg(not(target_arch = "wasm32"))]
use env_logger::Env;

#[cfg(target_arch = "wasm32")]
use log;
#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;

use vn_vttrpg_window::{init_with_logic, DefaultStateLogic};

pub fn init() -> anyhow::Result<()> {

    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    init_logging();
    log::info!("Logging was initialized!");

    log::info!("Initializing Application!");
    init_with_logic(DefaultStateLogic::default())?;

    log::info!("Application terminated!");
    Ok(())
}

pub fn init_logging() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let env = Env::default()
            .filter_or("MY_LOG_LEVEL", "Debug")
            .write_style_or("MY_LOG_STYLE", "always");
        env_logger::init_from_env(env);
    }

    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap();
    }
}