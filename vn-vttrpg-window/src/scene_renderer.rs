use crate::graphics::{GraphicsContext, VertexDescription};
use crate::pipeline_builder::PipelineBuilder;
use crate::primitives::{_TexturePrimitive, BoxPrimitive, Globals, QUAD_VERTICES, Vertex};
use crate::resource_manager::ResourceManager;
use crate::{Renderer, Texture, TextureDescriptor};
use std::sync::Arc;
use wgpu::include_wgsl;
use wgpu::util::DeviceExt;

struct GlobalResources {
    quad_vertex_buffer: wgpu::Buffer,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
}

impl GlobalResources {
    fn set<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_bind_group(0, &self.globals_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
    }
}

struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layouts: Vec<wgpu::BindGroupLayout>,
}

pub struct SceneRenderer {
    resource_manager: Arc<ResourceManager>,
    globals: GlobalResources,
    box_pipeline: Pipeline,
    texture_pipeline: Pipeline,
}

impl SceneRenderer {
    pub fn new(
        graphics_context: Arc<GraphicsContext>,
        resource_manager: Arc<ResourceManager>,
    ) -> Self {
        let device = graphics_context.device();

        let globals = {
            let config = graphics_context.config.borrow();
            Globals {
                resolution: [config.width as f32, config.height as f32],
            }
        };

        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Globals Buffer"),
            contents: bytemuck::cast_slice(&[globals]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let globals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let box_pipeline = PipelineBuilder::new(
            graphics_context.device(),
            graphics_context.config.borrow().format,
        )
        .label("Box Pipeline")
        .shader(&box_shader)
        .blend(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        })
        .add_vertex_layout(Vertex::vertex_description(
            None,
            None,
            wgpu::VertexStepMode::Vertex,
        ))
        .add_vertex_layout(BoxPrimitive::vertex_description(
            Some(Globals::location_count()),
            None,
            wgpu::VertexStepMode::Instance,
        ))
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

        let texture_pipeline = PipelineBuilder::new(
            graphics_context.device(),
            graphics_context.config.borrow().format,
        )
        .label("Texture Pipeline")
        .shader(&texture_shader)
        .blend(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        })
        .add_vertex_layout(Vertex::vertex_description(
            None,
            None,
            wgpu::VertexStepMode::Vertex,
        ))
        .add_vertex_layout(_TexturePrimitive::vertex_description(
            Some(Globals::location_count()),
            None,
            wgpu::VertexStepMode::Instance,
        ))
        .add_bind_group_layout(&globals_bind_group_layout)
        .add_bind_group_layout(&texture_bind_group_layout)
        .build()
        .expect("Failed to build texture pipeline");

        let quad_vertex_buffer =
            graphics_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Quad Vertex Buffer"),
                    contents: bytemuck::cast_slice(&QUAD_VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        Self {
            resource_manager,
            globals: GlobalResources {
                quad_vertex_buffer,
                globals_buffer,
                globals_bind_group,
            },
            box_pipeline: Pipeline {
                pipeline: box_pipeline,
                bind_group_layouts: vec![globals_bind_group_layout.clone()],
            },
            texture_pipeline: Pipeline {
                pipeline: texture_pipeline,
                bind_group_layouts: vec![globals_bind_group_layout, texture_bind_group_layout],
            },
        }
    }

    fn update_globals(&self, graphics_context: &GraphicsContext) {
        let globals = {
            let config = graphics_context.config.borrow();
            Globals {
                resolution: [config.width as f32, config.height as f32],
            }
        };
        graphics_context.queue().write_buffer(
            &self.globals.globals_buffer,
            0,
            bytemuck::cast_slice(&[globals]),
        );
    }

    fn render_boxes<'a>(
        &'a self,
        graphics_context: &GraphicsContext,
        render_pass: &mut wgpu::RenderPass<'a>,
        boxes: &[BoxPrimitive],
    ) {
        if boxes.is_empty() {
            return;
        }

        render_pass.set_pipeline(&self.box_pipeline.pipeline);
        self.globals.set(render_pass);

        let instance_buffer =
            graphics_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Box Instance Buffer"),
                    contents: bytemuck::cast_slice(boxes),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.draw(0..6, 0..boxes.len() as u32);
    }

    fn render_images<'a>(
        &'a self,
        graphics_context: &GraphicsContext,
        render_pass: &mut wgpu::RenderPass<'a>,
        images: &[crate::primitives::ImagePrimitive],
    ) {
        if images.is_empty() {
            return;
        }

        render_pass.set_pipeline(&self.texture_pipeline.pipeline);
        self.globals.set(render_pass);

        // Group by texture to minimize bind group changes and buffer creation
        let mut current_texture: Option<Arc<Texture>> = None;
        let mut batch = Vec::new();

        for image in images {
            let resolved = self.resolve_texture(&image.texture);

            if let Some(texture) = resolved {
                if let Some(ref current) = current_texture {
                    if !Arc::ptr_eq(current, &texture) {
                        self.draw_texture_batch(graphics_context, render_pass, current, &mut batch);
                        batch.clear();
                        current_texture = Some(texture);
                    }
                } else {
                    current_texture = Some(texture);
                }
                batch.push(image.to_texture_primitive());
            }
        }

        if let Some(ref current) = current_texture {
            self.draw_texture_batch(graphics_context, render_pass, current, &mut batch);
        }
    }

    fn render_texts<'a>(
        &'a self,
        graphics_context: &GraphicsContext,
        render_pass: &mut wgpu::RenderPass<'a>,
        texts: &[crate::primitives::TextPrimitive],
    ) {
        if texts.is_empty() {
            return;
        }

        render_pass.set_pipeline(&self.texture_pipeline.pipeline);
        self.globals.set(render_pass);

        let mut current_texture: Option<Arc<Texture>> = None;
        let mut batch = Vec::new();

        for text in texts {
            let resolved = self.resource_manager.get_or_render_text(
                graphics_context,
                &text.text,
                &text.font,
                text.font_size,
            );

            if let Some(texture) = resolved {
                if let Some(ref current) = current_texture {
                    if !Arc::ptr_eq(current, &texture) {
                        self.draw_texture_batch(graphics_context, render_pass, current, &mut batch);
                        batch.clear();
                        current_texture = Some(texture);
                    }
                } else {
                    current_texture = Some(texture);
                }
                batch.push(text.to_texture_primitive());
            }
        }

        if let Some(ref current) = current_texture {
            self.draw_texture_batch(graphics_context, render_pass, current, &mut batch);
        }
    }

    fn resolve_texture(&self, descriptor: &TextureDescriptor) -> Option<Arc<Texture>> {
        match descriptor {
            TextureDescriptor::Name(name) => self.resource_manager.get_texture(name),
            TextureDescriptor::Path(path) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let name = path.to_string_lossy();
                    if let Some(tex) = self.resource_manager.get_texture(&name) {
                        Some(tex)
                    } else {
                        match self.resource_manager.load_texture_from_path(&name, path) {
                            Ok(tex) => Some(tex),
                            Err(e) => {
                                log::error!("Failed to load texture from path {:?}: {}", path, e);
                                None
                            }
                        }
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    log::error!("Path loading not supported on WASM in render loop");
                    None
                }
            }
            TextureDescriptor::Bytes { name, bytes } => {
                if let Some(tex) = self.resource_manager.get_texture(name) {
                    Some(tex)
                } else {
                    match self.resource_manager.load_texture_from_bytes(name, bytes) {
                        Ok(tex) => Some(tex),
                        Err(e) => {
                            log::error!("Failed to load texture from bytes {}: {}", name, e);
                            None
                        }
                    }
                }
            }
        }
    }

    fn draw_texture_batch<'a>(
        &'a self,
        graphics_context: &GraphicsContext,
        render_pass: &mut wgpu::RenderPass<'a>,
        texture: &Arc<Texture>,
        batch: &mut Vec<_TexturePrimitive>,
    ) {
        if batch.is_empty() {
            return;
        }

        let bind_group = graphics_context
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Texture Bind Group"),
                layout: &self.texture_pipeline.bind_group_layouts[1],
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

        let instance_buffer =
            graphics_context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Texture Instance Buffer"),
                    contents: bytemuck::cast_slice(batch),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        render_pass.set_bind_group(1, &bind_group, &[]);
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.draw(0..6, 0..batch.len() as u32);
        batch.clear();
    }
}

impl Renderer for SceneRenderer {
    type RenderTarget = crate::scene::Scene;

    fn render(
        &mut self,
        graphics_context: &GraphicsContext,
        scene: &Self::RenderTarget,
    ) -> Result<(), wgpu::SurfaceError> {
        let (output, view, mut encoder) = Self::begin_render_frame(graphics_context)?;
        self.update_globals(graphics_context);

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

            for layer in scene.layers() {
                self.render_boxes(graphics_context, &mut render_pass, &layer.boxes);
                self.render_images(graphics_context, &mut render_pass, &layer.images);
                self.render_texts(graphics_context, &mut render_pass, &layer.texts);
            }
        }

        graphics_context
            .queue()
            .submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
