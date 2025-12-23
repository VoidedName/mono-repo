use crate::graphics::GraphicsContext;
use wgpu::include_wgsl;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};

pub trait StateLogic: Sized + 'static {
    #[allow(async_fn_in_trait)]
    async fn new_from_graphics_context(graphics_context: &GraphicsContext) -> anyhow::Result<Self>;

    #[allow(unused_variables)]
    fn handle_key(&self, event_loop: &ActiveEventLoop, event: &KeyEvent) {}

    /// Update will always be called before [`render`](Self::render)
    ///
    /// The intent of this split is that update may be called multiple times to update
    /// state, progress simulations, etc. independently of the expensive rendering call.
    #[allow(unused_variables)]
    fn update(&mut self) {}

    /// ;)
    #[allow(unused_variables)]
    fn render(&mut self, graphics_context: &GraphicsContext) -> Result<(), wgpu::SurfaceError> {
        Ok(())
    }
}

pub struct DefaultStateLogic {
    render_pipeline: wgpu::RenderPipeline,
}

impl StateLogic for DefaultStateLogic {
    async fn new_from_graphics_context(graphics_context: &GraphicsContext) -> anyhow::Result<Self> {
        let shader = graphics_context
            .device
            .create_shader_module(include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            graphics_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    immediate_size: 0,
                });

        let render_pipeline =
            graphics_context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: graphics_context.config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview_mask: None,
                    cache: None,
                });

        Ok(Self { render_pipeline })
    }

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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
