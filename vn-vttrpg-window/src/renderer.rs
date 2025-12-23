use crate::graphics::GraphicsContext;

pub trait Renderer {
    fn render<F>(&self, graphics_context: &GraphicsContext, draw_fn: F) -> Result<(), wgpu::SurfaceError>
    where
        F: FnMut(&mut wgpu::RenderPass);
}

pub struct WgpuRenderer {}

impl WgpuRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderer for WgpuRenderer {
    fn render<F>(&self, graphics_context: &GraphicsContext, mut draw_fn: F) -> Result<(), wgpu::SurfaceError>
    where
        F: FnMut(&mut wgpu::RenderPass),
    {
        let output = graphics_context.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = graphics_context.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            draw_fn(&mut render_pass);
        }

        graphics_context.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
