use crate::graphics::{GraphicsContext, VertexDescription, WgpuContext};
use crate::pipeline_builder::PipelineBuilder;
use crate::primitives::{
    _TexturePrimitive, BoxPrimitive, Globals, PrimitiveProperties, QUAD_VERTICES, Vertex,
};
use crate::resource_manager::ResourceManager;
use crate::texture::TextureId;
use crate::{GlyphInstance, ImagePrimitive, Renderer, TextPrimitive, Texture};
use similar::DiffOp;
use similar::algorithms::{Capture, Compact, Replace};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Index, Range};
use std::rc::Rc;
use vn_scene::{CloneableScene, Rect, Scene};
use wgpu::{include_wgsl, CommandEncoder};
use wgpu::util::DeviceExt;
use vn_ui::SceneRendererHook;

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

pub struct SceneRenderer<S: CloneableScene> {
    resource_manager: Rc<ResourceManager>,
    globals: GlobalResources,
    box_pipeline: Pipeline,
    texture_pipeline: Pipeline,
    instance_buffer: RefCell<wgpu::Buffer>,
    instance_buffer_capacity: Cell<usize>,
    instance_buffer_offset: Cell<usize>,
    box_instance_buffer: RefCell<wgpu::Buffer>,
    box_instance_buffer_capacity: Cell<usize>,
    box_instance_buffer_offset: Cell<usize>,
    batch: RefCell<Vec<_TexturePrimitive>>,
    pub graphics_context: Rc<GraphicsContext>,
    phantom: std::marker::PhantomData<S>,
}

impl<S: CloneableScene> SceneRenderer<S> {
    fn update_globals(&self, wgpu: &WgpuContext, target_size: (u32, u32)) {
        let globals = Globals {
            resolution: [target_size.0 as f32, target_size.1 as f32],
        };
        wgpu.queue.write_buffer(
            &self.globals.globals_buffer,
            0,
            bytemuck::cast_slice(&[globals]),
        );
    }

    fn render_boxes<'a>(
        &'a self,
        wgpu: &WgpuContext,
        render_pass: &mut wgpu::RenderPass<'a>,
        boxes: &[BoxPrimitive],
    ) {
        if boxes.is_empty() {
            return;
        }

        render_pass.set_pipeline(&self.box_pipeline.pipeline);
        self.globals.set(render_pass);

        let current_offset = self.box_instance_buffer_offset.get();
        let needed_capacity = current_offset + boxes.len();

        if needed_capacity > self.box_instance_buffer_capacity.get() {
            self.box_instance_buffer_capacity
                .set(needed_capacity.next_power_of_two());
            *self.box_instance_buffer.borrow_mut() =
                wgpu
                    .device
                    .create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Box Instance Buffer"),
                        size: (self.box_instance_buffer_capacity.get() * size_of::<BoxPrimitive>())
                            as u64,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
            self.box_instance_buffer_offset.set(0);
        }

        let offset_bytes =
            (self.box_instance_buffer_offset.get() * size_of::<BoxPrimitive>()) as u64;

        wgpu.queue.write_buffer(
            &self.box_instance_buffer.borrow(),
            offset_bytes,
            bytemuck::cast_slice(boxes),
        );

        render_pass.set_vertex_buffer(1, self.box_instance_buffer.borrow().slice(offset_bytes..));
        render_pass.draw(0..6, 0..boxes.len() as u32);

        self.box_instance_buffer_offset
            .set(self.box_instance_buffer_offset.get() + boxes.len());
    }

    fn render_images<'a>(
        &'a self,
        wgpu: &WgpuContext,
        render_pass: &mut wgpu::RenderPass<'a>,
        images: &[ImagePrimitive],
    ) {
        if images.is_empty() {
            return;
        }

        render_pass.set_pipeline(&self.texture_pipeline.pipeline);
        self.globals.set(render_pass);

        // Group by texture to minimize bind group changes and buffer creation
        let mut current_texture: Option<Rc<Texture>> = None;
        let mut batch = self.batch.borrow_mut();
        batch.clear();

        // todo: use the same batching as in text rendering

        for image in images {
            let resolved = self.resolve_texture(image.texture.clone());

            if let Some(texture) = resolved {
                if let Some(ref current) = current_texture {
                    if !Rc::ptr_eq(current, &texture) {
                        self.draw_texture_batch(wgpu, render_pass, current, &mut batch);
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
            self.draw_texture_batch(wgpu, render_pass, current, &mut batch);
        }
    }

    fn render_texts<'a>(
        &'a self,
        wgpu: &WgpuContext,
        render_pass: &mut wgpu::RenderPass<'a>,
        texts: &[TextPrimitive],
    ) {
        if texts.is_empty() {
            return;
        }

        render_pass.set_pipeline(&self.texture_pipeline.pipeline);
        self.globals.set(render_pass);

        // use a texture atlas instead: this is already much, much faster than drawing each glyph individually
        // but it scales with the number of distinct glyphs while an atlas is constant.

        // we can batch the glyphs like this because we have layers. Text that is rendered overlapping on
        // the same layer will have "undefined" behaviour.
        let mut batches = HashMap::<TextureId, (Rc<Texture>, Vec<_TexturePrimitive>)>::new();
        for text in texts {
            for glyph in &text.glyphs {
                let texture = self.resolve_texture(glyph.texture.clone());
                if texture.is_none() {
                    todo!(
                        "Implement FallBack Texture: Missing texture {:?}",
                        glyph.texture
                    );
                }

                let texture = texture.unwrap();

                batches
                    .entry(glyph.texture.clone())
                    // todo: i could do the texture lookup in the batch draw call
                    .or_insert_with(|| (texture.clone(), Vec::new()))
                    .1
                    .push({
                        let mut common = text.common;
                        common.transform.translation[0] += glyph.position[0];
                        common.transform.translation[1] += glyph.position[1];

                        _TexturePrimitive {
                            common,
                            uv_rect: glyph.uv_rect,
                            size: glyph.size,
                            tint: text.tint,
                        }
                    });
            }
        }

        let mut batch = self.batch.borrow_mut();
        for (_, (texture, mut b)) in batches.into_iter() {
            batch.clear();
            batch.append(&mut b);
            self.draw_texture_batch(wgpu, render_pass, &texture, &mut batch);
        }
    }

    fn resolve_texture(&self, descriptor: TextureId) -> Option<Rc<Texture>> {
        self.resource_manager.get_texture(descriptor)
    }

    fn draw_texture_batch<'a>(
        &'a self,
        wgpu: &WgpuContext,
        render_pass: &mut wgpu::RenderPass<'a>,
        texture: &Rc<Texture>,
        batch: &mut Vec<_TexturePrimitive>,
    ) {
        if batch.is_empty() {
            return;
        }

        let current_offset = self.instance_buffer_offset.get();
        let needed_capacity = current_offset + batch.len();

        if needed_capacity > self.instance_buffer_capacity.get() {
            self.instance_buffer_capacity
                .set(needed_capacity.next_power_of_two());
            *self.instance_buffer.borrow_mut() =
                wgpu
                    .device
                    .create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Instance Buffer"),
                        size: (self.instance_buffer_capacity.get() * size_of::<_TexturePrimitive>())
                            as u64,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
            self.instance_buffer_offset.set(0);
        }

        let offset_bytes =
            (self.instance_buffer_offset.get() * size_of::<_TexturePrimitive>()) as u64;

        wgpu.queue.write_buffer(
            &self.instance_buffer.borrow(),
            offset_bytes,
            bytemuck::cast_slice(batch),
        );

        let bind_group = wgpu
            .device
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

        render_pass.set_bind_group(1, &bind_group, &[]);
        render_pass.set_vertex_buffer(1, self.instance_buffer.borrow().slice(offset_bytes..));
        render_pass.draw(0..6, 0..batch.len() as u32);

        self.instance_buffer_offset
            .set(self.instance_buffer_offset.get() + batch.len());
        batch.clear();
    }
}

fn diff<Old, New>(
    old: &Old,
    old_range: Range<usize>,
    new: &New,
    new_range: Range<usize>,
) -> Vec<DiffOp>
where
    Old: Index<usize> + ?Sized,
    New: Index<usize> + ?Sized,
    Old::Output: Hash + Eq,
    New::Output: PartialEq<Old::Output> + Hash + Eq,
{
    let mut d = Compact::new(Replace::new(Capture::new()), old, new);
    similar::algorithms::lcs::diff(&mut d, old, old_range, new, new_range).unwrap();
    d.into_inner().into_inner().into_ops()
}

fn padded_zip<T: Default + Clone>(left: Vec<T>, right: Vec<T>) -> impl Iterator<Item = (T, T)> {
    let max_len = left.len().max(right.len());
    left.into_iter()
        .chain(std::iter::repeat(Default::default()))
        .take(max_len)
        .zip(
            right
                .into_iter()
                .chain(std::iter::repeat(Default::default()))
                .take(max_len),
        )
}

fn unified_clip_rect<T, F>(start: Option<Rect>, data: &[T], rect: F) -> Option<Rect>
where
    F: Fn(&T) -> Rect,
{
    data.iter().fold(start, |acc, x| {
        acc.map(|r| Rect::union(&r, &rect(x))).or(Some(rect(x)))
    })
}

impl<S: CloneableScene> Renderer for SceneRenderer<S> {
    type RenderTarget = S;

    fn new(graphics_context: Rc<GraphicsContext>, resource_manager: Rc<ResourceManager>) -> Self {
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

        let instance_buffer_capacity = 1024;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (instance_buffer_capacity * size_of::<_TexturePrimitive>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let box_instance_buffer_capacity = 1024;
        let box_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Box Instance Buffer"),
            size: (box_instance_buffer_capacity * size_of::<BoxPrimitive>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            phantom: Default::default(),
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
            instance_buffer: RefCell::new(instance_buffer),
            instance_buffer_capacity: Cell::new(instance_buffer_capacity),
            instance_buffer_offset: Cell::new(0),
            box_instance_buffer: RefCell::new(box_instance_buffer),
            box_instance_buffer_capacity: Cell::new(box_instance_buffer_capacity),
            box_instance_buffer_offset: Cell::new(0),
            batch: RefCell::new(Vec::new()),
            graphics_context: graphics_context.clone(),
        }
    }

    fn render(
        &mut self,
        wgpu: &WgpuContext,
        scene: &Self::RenderTarget,
        target_view: &wgpu::TextureView,
        target_size: (u32, u32),
        encoder: &mut CommandEncoder,
    ) -> Result<(), wgpu::SurfaceError> {
        let scene = scene.clone();
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            self.update_globals(wgpu, target_size);

            self.instance_buffer_offset.set(0);
            self.box_instance_buffer_offset.set(0);

            for layer in scene.layers() {
                self.render_boxes(
                    wgpu,
                    &mut render_pass,
                    &layer
                        .boxes
                        .iter()
                        .map(|b| BoxPrimitive {
                            common: PrimitiveProperties {
                                transform: b.transform,
                                clip_area: b.clip_rect,
                            },
                            size: b.size,
                            color: b.color,
                            border_color: b.border_color,
                            border_thickness: b.border_thickness,
                            corner_radius: b.border_radius,
                        })
                        .collect::<Vec<_>>(),
                );
                self.render_images(
                    wgpu,
                    &mut render_pass,
                    &layer
                        .images
                        .iter()
                        .map(|i| ImagePrimitive {
                            common: PrimitiveProperties {
                                transform: i.transform,
                                clip_area: i.clip_rect,
                            },
                            size: i.size,
                            uv_rect: i.uv_rect,
                            texture: i.texture_id.clone(),
                            tint: i.tint,
                        })
                        .collect::<Vec<_>>(),
                );
                self.render_texts(
                    wgpu,
                    &mut render_pass,
                    &layer
                        .texts
                        .iter()
                        .map(|t| TextPrimitive {
                            common: PrimitiveProperties {
                                transform: t.transform,
                                clip_area: t.clip_rect,
                            },
                            glyphs: t
                                .glyphs
                                .iter()
                                .map(|g| GlyphInstance {
                                    texture: g.texture_id.clone(),
                                    position: g.position,
                                    size: g.size,
                                    uv_rect: g.uv_rect,
                                })
                                .collect(),
                            tint: t.tint,
                        })
                        .collect::<Vec<_>>(),
                );
            }
        }

        Ok(())
    }
}

impl<S: CloneableScene> SceneRendererHook for SceneRenderer<S> {
    fn render_to_texture(
        &self,
        scene: &dyn Scene,
        size: (f32, f32),
        previous: Option<TextureId>,
    ) -> TextureId {
        let (width, height) = (size.0 as u32, size.1 as u32);
        
        // Find or create a texture to render into
        let texture = if let Some(id) = previous 
            && let Some(tex) = self.resource_manager.get_texture(id.clone()) 
            && tex.size == (width, height) {
            tex
        } else {
            let tex = Rc::new(Texture::create_render_target(
                &self.graphics_context.wgpu.device,
                (width, height),
                Some("Baked Texture"),
            ));
            self.resource_manager.add_texture(tex.clone());
            tex
        };
        
        let mut encoder = self.graphics_context.wgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Bake Scene Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bake Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            // Update globals for this sub-scene size
            let globals = Globals {
                resolution: [size.0, size.1],
            };
            self.graphics_context.wgpu.queue.write_buffer(
                &self.globals.globals_buffer,
                0,
                bytemuck::cast_slice(&[globals]),
            );

            self.instance_buffer_offset.set(0);
            self.box_instance_buffer_offset.set(0);


            for layer in scene.layers() {
                self.render_boxes(
                    &self.graphics_context.wgpu,
                    &mut render_pass,
                    &layer
                        .boxes
                        .iter()
                        .map(|b| BoxPrimitive {
                            common: PrimitiveProperties {
                                transform: b.transform,
                                clip_area: b.clip_rect,
                            },
                            size: b.size,
                            color: b.color,
                            border_color: b.border_color,
                            border_thickness: b.border_thickness,
                            corner_radius: b.border_radius,
                        })
                        .collect::<Vec<_>>(),
                );
                self.render_images(
                    &self.graphics_context.wgpu,
                    &mut render_pass,
                    &layer
                        .images
                        .iter()
                        .map(|i| ImagePrimitive {
                            common: PrimitiveProperties {
                                transform: i.transform,
                                clip_area: i.clip_rect,
                            },
                            size: i.size,
                            uv_rect: i.uv_rect,
                            texture: i.texture_id.clone(),
                            tint: i.tint,
                        })
                        .collect::<Vec<_>>(),
                );
                self.render_texts(
                    &self.graphics_context.wgpu,
                    &mut render_pass,
                    &layer
                        .texts
                        .iter()
                        .map(|t| TextPrimitive {
                            common: PrimitiveProperties {
                                transform: t.transform,
                                clip_area: t.clip_rect,
                            },
                            glyphs: t
                                .glyphs
                                .iter()
                                .map(|g| GlyphInstance {
                                    texture: g.texture_id.clone(),
                                    position: g.position,
                                    size: g.size,
                                    uv_rect: g.uv_rect,
                                })
                                .collect(),
                            tint: t.tint,
                        })
                        .collect::<Vec<_>>(),
                );
            }
        }

        self.graphics_context.wgpu.queue.submit(std::iter::once(encoder.finish()));

        texture.id.clone()
    }
}