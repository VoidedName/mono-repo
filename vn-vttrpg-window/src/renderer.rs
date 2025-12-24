use crate::graphics::{GraphicsContext, VertexDescription};
use crate::pipeline_builder::PipelineBuilder;
use crate::primitives::{BoxPrimitive, TexturePrimitive};
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

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    resolution: [f32; 2],
    _padding: [f32; 2],
}

pub struct WgpuRenderer {
    box_pipeline: wgpu::RenderPipeline,
    texture_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    quad_vertex_buffer: wgpu::Buffer,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
}

impl WgpuRenderer {
    pub fn new(graphics_context: &GraphicsContext) -> Self {
        let device = graphics_context.device();

        let globals = Globals {
            resolution: [graphics_context.config.width as f32, graphics_context.config.height as f32],
            _padding: [0.0; 2],
        };

        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Globals Buffer"),
            contents: bytemuck::cast_slice(&[globals]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let globals_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Globals Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Globals Bind Group"),
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let box_shader = graphics_context
            .device()
            .create_shader_module(include_wgsl!("shaders\\box_shader.wgsl"));

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
            .add_bind_group_layout(&globals_bind_group_layout)
            .build()
            .expect("Failed to build box pipeline");

        let texture_shader = graphics_context
            .device()
            .create_shader_module(include_wgsl!("shaders\\texture_shader.wgsl"));

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let texture_pipeline = PipelineBuilder::new(graphics_context.device(), graphics_context.config.format)
            .label("Texture Pipeline")
            .shader(&texture_shader)
            .add_vertex_layout(crate::graphics::VertexLayout {
                array_stride: size_of::<[f32; 2]>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: vec![wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }],
            })
            .add_vertex_layout(TexturePrimitive::vertex_description(Some(1), None, wgpu::VertexStepMode::Instance))
            .add_bind_group_layout(&globals_bind_group_layout)
            .add_bind_group_layout(&texture_bind_group_layout)
            .build()
            .expect("Failed to build texture pipeline");

        let quad_vertices: [[f32; 2]; 6] = [
            [0.0, 0.0], [0.0, 1.0], [1.0, 0.0],
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0],
        ];

        let quad_vertex_buffer = graphics_context.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&quad_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            box_pipeline,
            texture_pipeline,
            texture_bind_group_layout,
            quad_vertex_buffer,
            globals_buffer,
            globals_bind_group,
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

        let globals = Globals {
            resolution: [graphics_context.config.width as f32, graphics_context.config.height as f32],
            _padding: [0.0; 2],
        };
        graphics_context.queue().write_buffer(&self.globals_buffer, 0, bytemuck::cast_slice(&[globals]));

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

            for layer in &scene.layers {
                if !layer.boxes.is_empty() {
                    render_pass.set_pipeline(&self.box_pipeline);
                    render_pass.set_bind_group(0, &self.globals_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));

                    let instance_buffer = graphics_context.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Box Instance Buffer"),
                        contents: bytemuck::cast_slice(&layer.boxes),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                    render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                    render_pass.draw(0..6, 0..layer.boxes.len() as u32);
                }

                if !layer.images.is_empty() {
                    render_pass.set_pipeline(&self.texture_pipeline);
                    render_pass.set_bind_group(0, &self.globals_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));

                    // Group by texture to minimize bind group changes and buffer creation
                    // For now, let's just do it simple: one draw call per image or group by texture if they are contiguous
                    let mut current_texture: Option<std::sync::Arc<crate::Texture>> = None;
                    let mut batch = Vec::new();

                    let draw_batch = |texture: &std::sync::Arc<crate::Texture>, batch: &Vec<TexturePrimitive>, render_pass: &mut wgpu::RenderPass| {
                        let bind_group = graphics_context.device().create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Texture Bind Group"),
                            layout: &self.texture_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&texture.view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                                },
                            ],
                        });

                        let instance_buffer = graphics_context.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Texture Instance Buffer"),
                            contents: bytemuck::cast_slice(batch),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                        render_pass.set_bind_group(1, &bind_group, &[]);
                        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                        render_pass.draw(0..6, 0..batch.len() as u32);
                    };

                    for image in &layer.images {
                        if let Some(ref current) = current_texture {
                            if !std::sync::Arc::ptr_eq(current, &image.texture) {
                                draw_batch(current, &batch, &mut render_pass);
                                batch.clear();
                                current_texture = Some(image.texture.clone());
                            }
                        } else {
                            current_texture = Some(image.texture.clone());
                        }
                        batch.push(image.to_texture_primitive());
                    }

                    if let Some(ref current) = current_texture {
                        draw_batch(current, &batch, &mut render_pass);
                    }
                }
            }
        }

        graphics_context.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
