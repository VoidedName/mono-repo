pub mod logic;

use crate::logic::PlatformHooks;
pub use logic::MainLogic;
use std::rc::Rc;
use vn_wgpu_window::init_with_logic;

pub fn init(new_fn: Box<dyn PlatformHooks>) -> anyhow::Result<()> {
    log::info!("Initializing Tile Map Editor!");

    let new_fn = Rc::new(new_fn);

    init_with_logic(
        "Voided Names' Tile Map Editor".to_string(),
        (1280.0*2.0, 720.0*2.0),
        move |a, b| {
            let new_fn = new_fn.clone();
            async move { MainLogic::new(new_fn.clone(), a, b).await }
        },
    )?;

    log::info!("Tile Map Editor terminated!");
    Ok(())
}
