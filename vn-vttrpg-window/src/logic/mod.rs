use crate::graphics::GraphicsContext;
use crate::logic::vertex::{Vertex, INDICES, VERTICES};
use crate::texture::Texture;
use wgpu::include_wgsl;
use wgpu::util::DeviceExt;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};

mod vertex;

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
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_vertices: u32,
    pub diffuse_bind_group: wgpu::BindGroup,
}

impl StateLogic for DefaultStateLogic {
    async fn new_from_graphics_context(graphics_context: &GraphicsContext) -> anyhow::Result<Self> {
        let shader = graphics_context
            .device
            .create_shader_module(include_wgsl!("shader.wgsl"));

        let vertex_buffer =
            graphics_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            graphics_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let diffuse_bytes = include_bytes!("vn_dk_white_square_better_n.png");
        let diffuse_texture = Texture::from_bytes(
            &graphics_context.device,
            &graphics_context.queue,
            diffuse_bytes,
            Some("Diffuse Texture"),
        )?;

        let texture_bind_group_layout = graphics_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            }, wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler {
                    0: wgpu::SamplerBindingType::Filtering,
                },
                count: None,
            }],
        });

        let diffuse_bind_group = graphics_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
            }, wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
            }],
        });

        let render_pipeline_layout =
            graphics_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout],
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
                        buffers: &[Vertex::vertex_description()],
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

        Ok(Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_vertices: INDICES.len() as u32,
            diffuse_bind_group,
        })
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
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(0..self.num_vertices, 0, 0..1);
        }

        state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
