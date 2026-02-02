use crate::graphics::{GraphicsContext, VertexDescription};
use crate::pipeline_builder::PipelineBuilder;
use crate::primitives::{
    _TexturePrimitive, BoxPrimitive, Globals, PrimitiveProperties, QUAD_VERTICES, Vertex,
};
use crate::resource_manager::ResourceManager;
use crate::scene::WgpuScene;
use crate::texture::TextureId;
use crate::{GlyphInstance, ImagePrimitive, Renderer, TextPrimitive, Texture};
use similar::algorithms::{Capture, Compact, Replace};
use similar::DiffOp;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Index, Range};
use std::rc::Rc;
use vn_scene::{Rect, Scene};
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
    resource_manager: Rc<ResourceManager>,
    globals: GlobalResources,
    clear_pipeline: wgpu::RenderPipeline,
    box_pipeline: Pipeline,
    texture_pipeline: Pipeline,
    instance_buffer: RefCell<wgpu::Buffer>,
    instance_buffer_capacity: Cell<usize>,
    instance_buffer_offset: Cell<usize>,
    box_instance_buffer: RefCell<wgpu::Buffer>,
    box_instance_buffer_capacity: Cell<usize>,
    box_instance_buffer_offset: Cell<usize>,
    batch: RefCell<Vec<_TexturePrimitive>>,
    previous_scene: Option<Box<dyn Scene>>,
    backing_texture: RefCell<Option<Texture>>,
}

impl SceneRenderer {
    pub fn new(
        graphics_context: Rc<GraphicsContext>,
        resource_manager: Rc<ResourceManager>,
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

        let clear_pipeline = PipelineBuilder::new(
            graphics_context.device(),
            graphics_context.config.borrow().format,
        )
            .label("Clear Pipeline")
            .shader(&box_shader)
            .blend(wgpu::BlendState::REPLACE)
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
            .expect("Failed to build clear pipeline");

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
            previous_scene: None,
            resource_manager,
            globals: GlobalResources {
                quad_vertex_buffer,
                globals_buffer,
                globals_bind_group,
            },
            clear_pipeline,
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
            backing_texture: RefCell::new(None),
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

        let current_offset = self.box_instance_buffer_offset.get();
        let needed_capacity = current_offset + boxes.len();

        if needed_capacity > self.box_instance_buffer_capacity.get() {
            self.box_instance_buffer_capacity
                .set(needed_capacity.next_power_of_two());
            *self.box_instance_buffer.borrow_mut() =
                graphics_context
                    .device()
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

        graphics_context.queue().write_buffer(
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
        graphics_context: &GraphicsContext,
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
            self.draw_texture_batch(graphics_context, render_pass, &texture, &mut batch);
        }
    }

    fn resolve_texture(&self, descriptor: TextureId) -> Option<Rc<Texture>> {
        self.resource_manager.get_texture(descriptor)
    }

    fn draw_texture_batch<'a>(
        &'a self,
        graphics_context: &GraphicsContext,
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
                graphics_context
                    .device()
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

        graphics_context.queue().write_buffer(
            &self.instance_buffer.borrow(),
            offset_bytes,
            bytemuck::cast_slice(batch),
        );

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

fn padded_zip<T: Default + Clone>(left: Vec<T>, right: Vec<T>) -> impl Iterator<Item=(T, T)> {
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

impl Renderer for SceneRenderer {
    type RenderTarget = WgpuScene;

    fn render(
        &mut self,
        graphics_context: &GraphicsContext,
        scene: &Self::RenderTarget,
    ) -> Result<(), wgpu::SurfaceError> {
        // TODO: Consider caching and reusing previous render passes for identical scenes
        // TODO: Consider using some sort of scene diff to only rerender affected areas
        let scene = Box::new(scene.clone());

        let mut invalidated_rect = None;

        if let Some(previous_scene) = &self.previous_scene {
            if previous_scene.layers() == scene.layers() {
                return Ok(());
            }

            let left: Vec<_> = previous_scene.layers().iter().cloned().collect();
            let right: Vec<_> = scene.layers().iter().cloned().collect();

            macro_rules! compute_rect {
                ($old:expr, $new:expr) => {
                    for x in diff($old, 0..$old.len(), $new, 0..$new.len()) {
                        match x {
                            DiffOp::Delete {
                                old_index, old_len, ..
                            } => {
                                invalidated_rect = unified_clip_rect(
                                    invalidated_rect,
                                    &$old[old_index..old_index + old_len],
                                    |b| b.clip_rect,
                                );
                            }
                            DiffOp::Insert {
                                new_index, new_len, ..
                            } => {
                                invalidated_rect = unified_clip_rect(
                                    invalidated_rect,
                                    &$new[new_index..new_index + new_len],
                                    |b| b.clip_rect,
                                );
                            }
                            DiffOp::Replace {
                                old_index,
                                old_len,
                                new_index,
                                new_len,
                            } => {
                                invalidated_rect = unified_clip_rect(
                                    invalidated_rect,
                                    &$old[old_index..old_index + old_len],
                                    |b| b.clip_rect,
                                );
                                invalidated_rect = unified_clip_rect(
                                    invalidated_rect,
                                    &$new[new_index..new_index + new_len],
                                    |b| b.clip_rect,
                                );
                            }
                            _ => {}
                        }
                    }
                };
            }

            for (l, r) in padded_zip(left, right) {
                compute_rect!(&l.boxes, &r.boxes);
                compute_rect!(&l.images, &r.images);
                compute_rect!(&l.texts, &r.texts);
            }
        }

        let screen_rect = Rect {
            position: [0.0, 0.0],
            size: [scene.scene_size().0, scene.scene_size().1],
        };

        let mut invalidated_rect = invalidated_rect
            .map(|r| r.intersect(&screen_rect))
            .unwrap_or(screen_rect);

        invalidated_rect.position[0] = invalidated_rect.position[0].floor();
        invalidated_rect.position[1] = invalidated_rect.position[1].floor();
        invalidated_rect.size[0] = invalidated_rect.size[0].ceil();
        invalidated_rect.size[1] = invalidated_rect.size[1].ceil();

        let (output, _view, mut encoder) = Self::begin_render_frame(graphics_context)?;

        // Ensure backing texture exists and matches screen size
        let (width, height) = graphics_context.size();
        let mut backing_texture_ref = self.backing_texture.borrow_mut();
        if backing_texture_ref.as_ref().map(|t| t.size) != Some((width, height)) {
            *backing_texture_ref = Some(Texture::create_render_target(
                graphics_context.device(),
                (width, height),
                Some("Backing Texture"),
            ));
        }
        let backing_texture = backing_texture_ref.as_ref().unwrap();

        self.update_globals(graphics_context);

        self.instance_buffer_offset.set(0);
        self.box_instance_buffer_offset.set(0);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Backing Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &backing_texture.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: if invalidated_rect == screen_rect || self.previous_scene.is_none() {
                            wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 })
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    },
                },
                )],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_scissor_rect(
                invalidated_rect.position[0] as u32,
                invalidated_rect.position[1] as u32,
                invalidated_rect.size[0].max(1.0) as u32,
                invalidated_rect.size[1].max(1.0) as u32,
            );

            if invalidated_rect != screen_rect && self.previous_scene.is_some() {
                render_pass.set_pipeline(&self.clear_pipeline);
                self.globals.set(&mut render_pass);

                let clear_box = BoxPrimitive {
                    common: PrimitiveProperties {
                        transform: vn_scene::Transform::DEFAULT,
                        clip_area: screen_rect,
                    },
                    size: screen_rect.size,
                    color: crate::primitives::color::Color::TRANSPARENT,
                    border_color: crate::primitives::color::Color::TRANSPARENT,
                    border_thickness: 0.0,
                    corner_radius: 0.0,
                };

                let offset = self.box_instance_buffer_offset.get();
                graphics_context.queue().write_buffer(
                    &self.box_instance_buffer.borrow(),
                    (offset * size_of::<BoxPrimitive>()) as u64,
                    bytemuck::cast_slice(&[clear_box]),
                );

                render_pass.set_vertex_buffer(1, self.box_instance_buffer.borrow().slice(..));
                render_pass.draw(
                    0..QUAD_VERTICES.len() as u32,
                    offset as u32..(offset + 1) as u32,
                );
                self.box_instance_buffer_offset.set(offset + 1);
            }

            for layer in scene.layers() {
                self.render_boxes(
                    graphics_context,
                    &mut render_pass,
                    &layer
                        .boxes
                        .iter()
                        .filter_map(|b| {
                            let intersection = invalidated_rect.intersect(&b.clip_rect);
                            let invalidated = (intersection.size[0] + intersection.size[1]) != 0.0;

                            if invalidated {
                                Some(BoxPrimitive {
                                    common: PrimitiveProperties {
                                        transform: b.transform,
                                        clip_area: intersection,
                                    },
                                    size: b.size,
                                    color: b.color,
                                    border_color: b.border_color,
                                    border_thickness: b.border_thickness,
                                    corner_radius: b.border_radius,
                                })
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>(),
                );
                self.render_images(
                    graphics_context,
                    &mut render_pass,
                    &layer
                        .images
                        .iter()
                        .filter_map(|i| {
                            let intersection = invalidated_rect.intersect(&i.clip_rect);
                            let invalidated = (intersection.size[0] + intersection.size[1]) != 0.0;

                            if invalidated {
                                Some(ImagePrimitive {
                                    common: PrimitiveProperties {
                                        transform: i.transform,
                                        clip_area: intersection,
                                    },
                                    size: i.size,
                                    uv_rect: i.uv_rect,
                                    texture: i.texture_id.clone(),
                                    tint: i.tint,
                                })
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>(),
                );
                self.render_texts(
                    graphics_context,
                    &mut render_pass,
                    &layer
                        .texts
                        .iter()
                        .filter_map(|t| {
                            let intersection = invalidated_rect.intersect(&t.clip_rect);
                            let invalidated = (intersection.size[0] + intersection.size[1]) != 0.0;

                            if invalidated {
                                Some(TextPrimitive {
                                    common: PrimitiveProperties {
                                        transform: t.transform,
                                        clip_area: intersection,
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
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>(),
                );
            }
        }

        // Copy backing texture to swapchain
        {
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &backing_texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: &output.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }

        self.previous_scene.replace(scene);

        graphics_context
            .queue()
            .submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
