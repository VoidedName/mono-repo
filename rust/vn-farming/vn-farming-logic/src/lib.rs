use std::path::Path;
use std::pin::Pin;
use crate::game::Game;
use std::rc::Rc;
use vn_scene::{CloneableScene, ConstructableScene};
use vn_wgpu_window::init_with_logic;

pub const UI_FONT: &str = "jetbrains-bold";
pub const UI_FONT_SIZE: f32 = 16.0;

pub mod game;

pub trait PlatformHooks: 'static {
    fn execute_async(&self, f: impl Future<Output = ()> + 'static);
    fn load_asset(
        &self,
        path: impl AsRef<Path>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>, String>>>>;
    fn logic_was_initialized(&self);
    fn exit(&self);
}

pub fn init<S: CloneableScene + ConstructableScene + 'static, P: PlatformHooks + 'static>(
    platform: P,
) -> anyhow::Result<()> {
    log::info!("Initializing Farming Game!");

    let platform = Rc::new(platform);

    init_with_logic(
        "Voided Names' Farming Game".to_string(),
        (1280.0 * 2.0, 720.0 * 2.0),
        move |dispatcher, gc, rm| {
            let platform = platform.clone();
            async move {
                {
                    let r = Game::<S, P>::new(Rc::new(dispatcher), platform.clone(), gc, rm).await;
                    platform.logic_was_initialized();
                    r
                }
            }
        },
    )?;

    log::info!("Tile Map Editor terminated!");
    Ok(())
}
