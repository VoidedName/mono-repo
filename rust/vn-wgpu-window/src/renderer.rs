use crate::GraphicsContext;
use crate::resource_manager::ResourceManager;
use std::rc::Rc;
use crate::graphics::WgpuContext;

/// A trait for types that can render a specific target using a [`GraphicsContext`].
pub trait Renderer {
    /// The type that this renderer can draw.
    type RenderTarget;

    fn new(context: Rc<GraphicsContext>, resource_manager: Rc<ResourceManager>) -> Self;

    /// Renders the target to the specified texture view.
    fn render(
        &mut self,
        wgpu: &WgpuContext,
        target: &Self::RenderTarget,
        target_view: &wgpu::TextureView,
        target_size: (u32, u32),
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), wgpu::SurfaceError>;

    /// Prepares the graphics context for a new frame, returning the surface texture, view, and encoder.
    fn begin_render_frame(
        graphics_context: &GraphicsContext,
    ) -> Result<
        (
            wgpu::SurfaceTexture,
            wgpu::TextureView,
            wgpu::CommandEncoder,
        ),
        wgpu::SurfaceError,
    > {
        let output = graphics_context.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder =
            graphics_context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        Ok((output, view, encoder))
    }
}
