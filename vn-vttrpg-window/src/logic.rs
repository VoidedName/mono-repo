use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use crate::graphics::GraphicsContext;

pub trait StateLogic: Sized + 'static {
    #[allow(unused_variables)]
    fn handle_key(&self, event_loop: &ActiveEventLoop, event: &KeyEvent) {}
    #[allow(unused_variables)]
    fn update(&mut self) {}
    #[allow(unused_variables)]
    fn render(&mut self, state: &GraphicsContext) -> Result<(), wgpu::SurfaceError> {
        Ok(())
    }
}

pub struct DefaultStateLogic;

impl StateLogic for DefaultStateLogic {
    fn handle_key(&self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        match (event.physical_key, event.state.is_pressed()) {
            (PhysicalKey::Code(KeyCode::Escape), true) => event_loop.exit(),
            _ => {
                log::info!("Key: {:?} State: {:?}", event.physical_key, event.state);
            }
        }
    }

    fn render(&mut self, state: &GraphicsContext) -> Result<(), wgpu::SurfaceError> {
        let output = state.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

impl Default for DefaultStateLogic {
    fn default() -> Self {
        Self
    }
}
