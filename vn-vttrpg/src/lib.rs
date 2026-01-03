pub mod logic;

use crate::logic::FileLoader;
pub use logic::MainLogic;
use std::rc::Rc;
use vn_window::init_with_logic;

pub fn init(new_fn: Box<dyn FileLoader>) -> anyhow::Result<()> {
    log::info!("Initializing Application!");

    let new_fn = Rc::new(new_fn);

    init_with_logic(move |a, b| {
        let new_fn = new_fn.clone();
        async move { MainLogic::new(new_fn.clone(), a, b).await }
    })?;

    log::info!("Application terminated!");
    Ok(())
}
