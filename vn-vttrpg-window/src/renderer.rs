use crate::graphics::{GraphicsContext, VertexDescription};
use crate::pipeline_builder::PipelineBuilder;
use crate::primitives::BoxPrimitive;
use wgpu::include_wgsl;
use wgpu::util::DeviceExt;

pub trait Renderer {
    type RenderTarget;

    fn render(
        &mut self,
        graphics_context: &GraphicsContext,
        scene: &Self::RenderTarget,
    ) -> Result<(), wgpu::SurfaceError>;
}

pub struct WgpuRenderer {
    box_pipeline: wgpu::RenderPipeline,
    quad_vertex_buffer: wgpu::Buffer,
}

impl WgpuRenderer {
    pub fn new(graphics_context: &GraphicsContext) -> Self {
        let box_shader = graphics_context
            .device()
            .create_shader_module(include_wgsl!("primitives\\box_shader.wgsl"));

        let box_pipeline = PipelineBuilder::new(graphics_context.device(), graphics_context.config.format)
            .label("Box Pipeline")
            .shader(&box_shader)
            .add_vertex_layout(crate::graphics::VertexLayout {
                array_stride: size_of::<[f32; 2]>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: vec![wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }],
            })
            .add_vertex_layout(BoxPrimitive::vertex_description(Some(1), None, wgpu::VertexStepMode::Instance))
            .build()
            .expect("Failed to build box pipeline");

        let quad_vertices: [[f32; 2]; 6] = [
            [0.0, 0.0], [1.0, 0.0], [0.0, 1.0],
            [0.0, 1.0], [1.0, 0.0], [1.0, 1.0],
        ];

        let quad_vertex_buffer = graphics_context.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&quad_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            box_pipeline,
            quad_vertex_buffer,
        }
    }
}

impl Renderer for WgpuRenderer {
    type RenderTarget = crate::scene::Scene;

    fn render(
        &mut self,
        graphics_context: &GraphicsContext,
        scene: &Self::RenderTarget,
    ) -> Result<(), wgpu::SurfaceError> {
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

            render_pass.set_pipeline(&self.box_pipeline);
            render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));

            for layer in &scene.layers {
                if !layer.boxes.is_empty() {
                    let instance_buffer = graphics_context.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Box Instance Buffer"),
                        contents: bytemuck::cast_slice(&layer.boxes),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                    render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                    render_pass.draw(0..6, 0..layer.boxes.len() as u32);
                }
            }
        }

        graphics_context.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
