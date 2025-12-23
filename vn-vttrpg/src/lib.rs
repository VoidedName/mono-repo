use vn_vttrpg_window::{init_with_logic, DefaultStateLogic};

pub fn init() -> anyhow::Result<()> {
    log::info!("Initializing Application!");
    init_with_logic::<DefaultStateLogic>()?;

    log::info!("Application terminated!");
    Ok(())
}