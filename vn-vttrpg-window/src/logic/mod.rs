use crate::graphics::GraphicsContext;
use crate::resource_manager::ResourceManager;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;

mod vertex;

pub trait StateLogic: Sized + 'static {
    #[allow(async_fn_in_trait)]
    async fn new_from_graphics_context(
        graphics_context: &GraphicsContext,
        resource_manager: &mut ResourceManager,
    ) -> anyhow::Result<Self>;

    #[allow(unused_variables)]
    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {}

    #[allow(unused_variables)]
    fn update(&mut self) {}

    #[allow(unused_variables)]
    fn draw(&self, render_pass: &mut wgpu::RenderPass) {}
}

pub struct DefaultStateLogic {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_vertices: u32,
    pub diffuse_bind_group: wgpu::BindGroup,
}

impl StateLogic for DefaultStateLogic {
    async fn new_from_graphics_context(
        graphics_context: &GraphicsContext,
        resource_manager: &mut ResourceManager,
    ) -> anyhow::Result<Self> {
        use crate::logic::vertex::{Vertex, INDICES, VERTICES};
        use wgpu::include_wgsl;
        use wgpu::util::DeviceExt;
        use crate::pipeline_builder::PipelineBuilder;

        let shader = graphics_context
            .device()
            .create_shader_module(include_wgsl!("shader.wgsl"));

        let vertex_buffer =
            graphics_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            graphics_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let diffuse_bytes = include_bytes!("vn_dk_white_square_better_n.png");
        let diffuse_texture = resource_manager.load_texture("vn_dk_white_square", diffuse_bytes)?;

        let texture_bind_group_layout = graphics_context.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let diffuse_bind_group = graphics_context.device().create_bind_group(&wgpu::BindGroupDescriptor {
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

        let render_pipeline = PipelineBuilder::new(graphics_context.device(), graphics_context.config.format)
            .label("Render Pipeline")
            .shader(&shader)
            .add_vertex_buffer(Vertex::vertex_description())
            .add_bind_group_layout(&texture_bind_group_layout)
            .build()?;

        Ok(Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_vertices: INDICES.len() as u32,
            diffuse_bind_group,
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

    fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.num_vertices, 0, 0..1);
    }
}
