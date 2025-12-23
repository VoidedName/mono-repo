pub mod app;
pub mod graphics;
pub mod logic;
pub mod rendering_context;
mod texture;

pub use app::App;
pub use graphics::GraphicsContext;
pub use logic::{DefaultStateLogic, StateLogic};
pub use rendering_context::RenderingContext;
pub use texture::Texture;

use winit::event_loop::EventLoop;

pub fn init() -> anyhow::Result<()> {
    init_with_logic::<DefaultStateLogic>()
}

pub fn init_with_logic<T: StateLogic>() -> anyhow::Result<()> {
    log::info!("Initializing window");

    let event_loop = EventLoop::<RenderingContext<T>>::with_user_event().build()?;
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );

    log::info!("Running the event loop!");
    event_loop.run_app(&mut app)?;

    Ok(())
}
