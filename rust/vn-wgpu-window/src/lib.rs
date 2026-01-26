pub mod app;
pub mod errors;
pub mod graphics;
pub mod logic;
pub mod pipeline_builder;
pub mod primitives;
mod renderer;
pub mod rendering_context;
pub mod resource_manager;
pub mod scene;
pub mod scene_renderer;
pub mod text;
pub use text::Glyph;
mod texture;

pub use app::App;
pub use graphics::GraphicsContext;
pub use logic::StateLogic;
pub use primitives::{
    _TexturePrimitive, BoxPrimitive, Color, Globals, GlyphInstance, ImagePrimitive, Rect,
    TextPrimitive, Transform,
};
pub use renderer::Renderer;
pub use rendering_context::RenderingContext;
pub use scene::WgpuScene;
pub use scene_renderer::SceneRenderer;
pub use texture::Texture;

use winit::event_loop::EventLoop;

pub fn init_with_logic<FNew, FRet, T: StateLogic<SceneRenderer>>(
    title: String,
    size: (f32, f32),
    new_fn: FNew,
) -> anyhow::Result<()>
where
    FNew: Fn(std::rc::Rc<GraphicsContext>, std::rc::Rc<resource_manager::ResourceManager>) -> FRet
        + 'static,
    FRet: Future<Output = anyhow::Result<T>>,
{
    log::info!("Initializing window");

    let event_loop = EventLoop::<RenderingContext<T>>::with_user_event().build()?;
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
        title,
        size,
        new_fn,
    );

    log::info!("Running the event loop!");
    event_loop.run_app(&mut app)?;

    Ok(())
}
