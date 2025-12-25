pub mod logic;
pub use logic::MainLogic;
use vn_vttrpg_window::init_with_logic;

pub fn init() -> anyhow::Result<()> {
    log::info!("Initializing Application!");
    init_with_logic::<MainLogic>()?;

    log::info!("Application terminated!");
    Ok(())
}
