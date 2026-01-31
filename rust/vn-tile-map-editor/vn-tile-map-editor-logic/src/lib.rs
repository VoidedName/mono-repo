pub mod logic;

use crate::logic::PlatformHooks;
pub use logic::MainLogic;
use std::rc::Rc;
use vn_wgpu_window::init_with_logic;

pub const UI_FONT: &str = "jetbrains-bold";
pub const UI_FONT_SIZE: f32 = 16.0;

pub fn init<P: PlatformHooks + 'static>(platform: P) -> anyhow::Result<()> {
    log::info!("Initializing Tile Map Editor!");

    let platform = Rc::new(platform);

    init_with_logic(
        "Voided Names' Tile Map Editor".to_string(),
        (1280.0 * 2.0, 720.0 * 2.0),
        move |a, b| {
            let platform = platform.clone();
            async move {
                {
                    let r = MainLogic::new(platform.clone(), a, b).await;
                    P::has_initialized();
                    r
                }
            }
        },
    )?;

    log::info!("Tile Map Editor terminated!");
    Ok(())
}
