pub mod app;
pub mod errors;
pub mod graphics;
pub mod input;
pub mod logic;
pub mod pipeline_builder;
pub mod primitives;
mod renderer;
pub mod rendering_context;
pub mod resource_manager;
pub mod scene;
pub mod scene_renderer;
pub mod text;
mod texture;

pub use app::App;
pub use graphics::GraphicsContext;
pub use logic::StateLogic;
pub use primitives::{
    _TexturePrimitive, BoxPrimitive, Color, Globals, ImagePrimitive, Rect, TextPrimitive, Transform,
};
pub use renderer::Renderer;
pub use rendering_context::RenderingContext;
pub use scene::{Layer, Scene};
pub use scene_renderer::SceneRenderer;
pub use texture::{Texture, TextureDescriptor};

use winit::event_loop::EventLoop;

pub fn init_with_logic<T: StateLogic<SceneRenderer>>() -> anyhow::Result<()> {
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
