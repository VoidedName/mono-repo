use crate::GraphicsContext;

/// A trait for types that can render a specific target using a [`GraphicsContext`].
pub trait Renderer {
    /// The type that this renderer can draw.
    type RenderTarget;

    /// Renders the target to the current surface.
    fn render(
        &mut self,
        graphics_context: &GraphicsContext,
        target: &Self::RenderTarget,
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
