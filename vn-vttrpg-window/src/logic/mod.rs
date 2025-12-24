use crate::graphics::{GraphicsContext, VertexDescription};
use crate::renderer::{Renderer, WgpuRenderer};
use crate::resource_manager::ResourceManager;
use std::f32::consts::PI;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;
use web_time::Instant;

mod vertex;

pub trait StateLogic<R: Renderer>: Sized + 'static {
    #[allow(async_fn_in_trait)]
    async fn new_from_graphics_context(
        graphics_context: &GraphicsContext,
        resource_manager: &mut ResourceManager,
    ) -> anyhow::Result<Self>;

    #[allow(unused_variables)]
    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {}

    #[allow(unused_variables)]
    fn update(&mut self) {}

    fn render_target(&self) -> R::RenderTarget;
}

pub struct DefaultStateLogic {
    pub diffuse_bind_group: wgpu::BindGroup,
    pub diffuse_texture: std::sync::Arc<crate::Texture>,
    application_start: Instant,
}

impl StateLogic<WgpuRenderer> for DefaultStateLogic {
    async fn new_from_graphics_context(
        graphics_context: &GraphicsContext,
        resource_manager: &mut ResourceManager,
    ) -> anyhow::Result<Self> {
        use crate::logic::vertex::INDICES;
        use crate::logic::vertex::VERTICES;
        use crate::logic::vertex::Vertex;
        use crate::pipeline_builder::PipelineBuilder;
        use wgpu::include_wgsl;
        use wgpu::util::DeviceExt;

        let shader = graphics_context
            .device()
            .create_shader_module(include_wgsl!("shader.wgsl"));

        let _vertex_buffer =
            graphics_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let _index_buffer =
            graphics_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let diffuse_bytes = include_bytes!("vn_dk_white_square_better_n.png");
        let diffuse_texture = resource_manager.load_texture("vn_dk_white_square", diffuse_bytes)?;

        let texture_bind_group_layout =
            graphics_context
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Texture Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler {
                                0: wgpu::SamplerBindingType::Filtering,
                            },
                            count: None,
                        },
                    ],
                });

        let diffuse_bind_group =
            graphics_context
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Bind Group"),
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                        },
                    ],
                });

        let _render_pipeline =
            PipelineBuilder::new(graphics_context.device(), graphics_context.config.format)
                .label("Render Pipeline")
                .shader(&shader)
                .add_vertex_layout(Vertex::vertex_description(
                    None,
                    None,
                    wgpu::VertexStepMode::Vertex,
                ))
                .add_bind_group_layout(&texture_bind_group_layout)
                .build()?;

        Ok(Self {
            diffuse_bind_group,
            diffuse_texture,
            application_start: Instant::now(),
        })
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        use winit::keyboard::{KeyCode, PhysicalKey};
        match (event.physical_key, event.state.is_pressed()) {
            (PhysicalKey::Code(KeyCode::Escape), true) => event_loop.exit(),
            _ => {
                log::info!("Key: {:?} State: {:?}", event.physical_key, event.state);
            }
        }
    }

    fn render_target(&self) -> crate::scene::Scene {
        use crate::primitives::{BoxPrimitive, Color, ImagePrimitive, PrimitiveProperties, Rect, Transform};
        let mut scene = crate::scene::Scene::new();
        scene.add_box(BoxPrimitive {
            common: PrimitiveProperties {
                transform: Transform {
                    translation: [200.0, 200.0],
                    rotation: self.application_start
                        .elapsed()
                        .as_secs_f32()
                        * 0.5
                        * PI,
                    scale: [1.0, 1.0],
                    origin: [0.5, 0.5],
                },
                clip_area: Rect::NO_CLIP,
            },
            size: [200.0, 150.0],
            color: Color::RED,
            border_color: Color::WHITE,
            border_thickness: 5.0,
            corner_radius: 10.0,
        });

        scene.add_image(ImagePrimitive {
            common: PrimitiveProperties {
                transform: Transform {
                    translation: [200.0, 200.0],
                    rotation: self.application_start
                        .elapsed()
                        .as_secs_f32()
                        * 0.5
                        * PI,
                    scale: [1.0, 1.0],
                    origin: [0.5, 0.5],
                },
                clip_area: Rect::NO_CLIP,
            },
            size: [100.0, 100.0],
            texture: self.diffuse_texture.clone(),
            tint: Color::WHITE,
        });

        scene
    }
}
