pub mod logic;

use crate::logic::PlatformHooks;
pub use logic::MainLogic;
use std::rc::Rc;
use vn_wgpu_window::init_with_logic;

pub fn init(new_fn: Box<dyn PlatformHooks>) -> anyhow::Result<()> {
    log::info!("Initializing Application!");

    let new_fn = Rc::new(new_fn);

    init_with_logic(
        // Note (Application title): This only sets it for native
        //  for web, adjust the title in the index.html file. Maybe at some point
        //  in time I will implement a platform independent hook
        "Voided Names' VTTRPG".to_string(),
        move |a, b| {
            let new_fn = new_fn.clone();
            async move { MainLogic::new(new_fn.clone(), a, b).await }
        },
    )?;

    log::info!("Application terminated!");
    Ok(())
}
