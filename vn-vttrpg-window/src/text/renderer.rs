use crate::graphics::{GraphicsContext, VertexDescription};
use crate::pipeline_builder::PipelineBuilder;
use crate::primitives::QUAD_VERTICES;
use crate::primitives::{Globals, Vertex};
use crate::text::Font;
use crate::texture::Texture;
use bytemuck::{Pod, Zeroable};
use ttf_parser::OutlineBuilder;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuSegment {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuGlyph {
    pub rect_min: [f32; 2],
    pub rect_max: [f32; 2],
    pub segment_start: u32,
    pub segment_count: u32,
}

pub struct TextRenderer {
    pipeline: wgpu::RenderPipeline,
    glyph_bind_group_layout: wgpu::BindGroupLayout,
    segment_bind_group_layout: wgpu::BindGroupLayout,

    quad_vertex_buffer: wgpu::Buffer,
    globals_buffer: wgpu::Buffer,
    glyph_buffer: wgpu::Buffer,
    segment_buffer: wgpu::Buffer,

    globals_bind_group: wgpu::BindGroup,
    glyph_bind_group: wgpu::BindGroup,
    segment_bind_group: wgpu::BindGroup,

    glyph_buffer_capacity: usize,
    segment_buffer_capacity: usize,
}

impl TextRenderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/text_shader.wgsl"));

        let globals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Text Globals Bind Group Layout"),
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

        let glyph_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Glyph Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let segment_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Segment Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline = PipelineBuilder::new(device, wgpu::TextureFormat::Rgba8UnormSrgb)
            .label("Text Pipeline")
            .shader(&shader)
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
            .add_bind_group_layout(&globals_bind_group_layout)
            .add_bind_group_layout(&glyph_bind_group_layout)
            .add_bind_group_layout(&segment_bind_group_layout)
            .build()
            .expect("Failed to build text pipeline");

        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Text Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Globals Buffer"),
            size: size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let glyph_buffer_capacity = 128;
        let glyph_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Glyph Buffer"),
            size: (glyph_buffer_capacity * size_of::<GpuGlyph>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let segment_buffer_capacity = 1024;
        let segment_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Segment Buffer"),
            size: (segment_buffer_capacity * size_of::<GpuSegment>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Globals Bind Group"),
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let glyph_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Glyph Bind Group"),
            layout: &glyph_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: glyph_buffer.as_entire_binding(),
            }],
        });

        let segment_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Segment Bind Group"),
            layout: &segment_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: segment_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            glyph_bind_group_layout,
            segment_bind_group_layout,
            quad_vertex_buffer,
            globals_buffer,
            glyph_buffer,
            segment_buffer,
            globals_bind_group,
            glyph_bind_group,
            segment_bind_group,
            glyph_buffer_capacity,
            segment_buffer_capacity,
        }
    }

    pub fn render_string(
        &mut self,
        graphics_context: &GraphicsContext,
        font: &Font,
        text: &str,
        font_size: f32,
    ) -> anyhow::Result<Texture> {
        let face = font
            .face()
            .map_err(|e| anyhow::anyhow!("Font parse error: {}", e))?;
        let scale = font_size / face.units_per_em() as f32;

        let mut current_x = 0.0;
        let mut glyph_instances = Vec::new();
        let mut all_segments = Vec::new();

        let ascender = face.ascender() as f32 * scale;

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for c in text.chars() {
            if let Some(glyph_id) = face.glyph_index(c) {
                let segment_start = all_segments.len() as u32;

                let mut collector = OutlineCollector::new([current_x, ascender], scale);
                face.outline_glyph(glyph_id, &mut collector);
                let segment_count = collector.segments.len() as u32;

                if let Some(bbox) = face.glyph_bounding_box(glyph_id) {
                    let r_min_x = current_x + bbox.x_min as f32 * scale;
                    let r_max_x = current_x + bbox.x_max as f32 * scale;
                    let r_min_y = ascender - bbox.y_max as f32 * scale;
                    let r_max_y = ascender - bbox.y_min as f32 * scale;

                    glyph_instances.push(GpuGlyph {
                        rect_min: [r_min_x, r_min_y],
                        rect_max: [r_max_x, r_max_y],
                        segment_start,
                        segment_count,
                    });

                    min_x = min_x.min(r_min_x);
                    min_y = min_y.min(r_min_y);
                    max_x = max_x.max(r_max_x);
                    max_y = max_y.max(r_max_y);
                }

                all_segments.extend(collector.segments);
                current_x += face.glyph_hor_advance(glyph_id).unwrap_or(0) as f32 * scale;
            }
        }

        if glyph_instances.is_empty() {
            return Ok(Texture::create_render_target(
                graphics_context.device(),
                (1, 1),
                Some("Empty Text"),
            ));
        }

        let width = (max_x - min_x).ceil() as u32 + 2;
        let height = (max_y - min_y).ceil() as u32 + 2;

        let offset_x = -min_x + 1.0;
        let offset_y = -min_y + 1.0;

        for glyph in &mut glyph_instances {
            glyph.rect_min[0] += offset_x;
            glyph.rect_min[1] += offset_y;
            glyph.rect_max[0] += offset_x;
            glyph.rect_max[1] += offset_y;
        }
        for seg in &mut all_segments {
            seg.p0[0] += offset_x;
            seg.p0[1] += offset_y;
            seg.p1[0] += offset_x;
            seg.p1[1] += offset_y;
        }

        let device = graphics_context.device();
        let queue = graphics_context.queue();

        // Resize buffers if necessary
        if glyph_instances.len() > self.glyph_buffer_capacity {
            self.glyph_buffer_capacity = glyph_instances.len().next_power_of_two();
            self.glyph_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Text Glyph Buffer"),
                size: (self.glyph_buffer_capacity * std::mem::size_of::<GpuGlyph>()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.glyph_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Text Glyph Bind Group"),
                layout: &self.glyph_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.glyph_buffer.as_entire_binding(),
                }],
            });
        }

        if all_segments.len() > self.segment_buffer_capacity {
            self.segment_buffer_capacity = all_segments.len().next_power_of_two();
            self.segment_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Text Segment Buffer"),
                size: (self.segment_buffer_capacity * std::mem::size_of::<GpuSegment>()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.segment_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Text Segment Bind Group"),
                layout: &self.segment_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.segment_buffer.as_entire_binding(),
                }],
            });
        }

        queue.write_buffer(
            &self.glyph_buffer,
            0,
            bytemuck::cast_slice(&glyph_instances),
        );
        queue.write_buffer(&self.segment_buffer, 0, bytemuck::cast_slice(&all_segments));

        let globals = Globals {
            resolution: [width as f32, height as f32],
        };
        queue.write_buffer(&self.globals_buffer, 0, bytemuck::cast_slice(&[globals]));

        let target_texture =
            Texture::create_render_target(device, (width, height), Some("Text Texture"));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Text Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Text Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.globals_bind_group, &[]);
            render_pass.set_bind_group(1, &self.glyph_bind_group, &[]);
            render_pass.set_bind_group(2, &self.segment_bind_group, &[]);
            render_pass.draw(0..6, 0..glyph_instances.len() as u32);
        }

        queue.submit(std::iter::once(encoder.finish()));

        Ok(target_texture)
    }
}

struct OutlineCollector {
    current_point: [f32; 2],
    first_point: [f32; 2],
    segments: Vec<GpuSegment>,
    offset: [f32; 2],
    scale: f32,
}

impl OutlineCollector {
    fn new(offset: [f32; 2], scale: f32) -> Self {
        Self {
            current_point: [0.0, 0.0],
            first_point: [0.0, 0.0],
            segments: Vec::new(),
            offset,
            scale,
        }
    }

    fn transform(&self, x: f32, y: f32) -> [f32; 2] {
        [
            self.offset[0] + x * self.scale,
            self.offset[1] - y * self.scale,
        ]
    }
}

impl OutlineBuilder for OutlineCollector {
    fn move_to(&mut self, x: f32, y: f32) {
        self.current_point = self.transform(x, y);
        self.first_point = self.current_point;
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.transform(x, y);
        self.segments.push(GpuSegment {
            p0: self.current_point,
            p1: p,
        });
        self.current_point = p;
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let p1 = self.transform(x1, y1);
        let p = self.transform(x, y);
        let p0 = self.current_point;

        let steps = 8;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let t_inv = 1.0 - t;
            let next_p = [
                t_inv * t_inv * p0[0] + 2.0 * t_inv * t * p1[0] + t * t * p[0],
                t_inv * t_inv * p0[1] + 2.0 * t_inv * t * p1[1] + t * t * p[1],
            ];
            self.segments.push(GpuSegment {
                p0: self.current_point,
                p1: next_p,
            });
            self.current_point = next_p;
        }
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let p1 = self.transform(x1, y1);
        let p2 = self.transform(x2, y2);
        let p = self.transform(x, y);
        let p0 = self.current_point;

        let steps = 12;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let t_inv = 1.0 - t;
            let next_p = [
                t_inv * t_inv * t_inv * p0[0]
                    + 3.0 * t_inv * t_inv * t * p1[0]
                    + 3.0 * t_inv * t * t * p2[0]
                    + t * t * t * p[0],
                t_inv * t_inv * t_inv * p0[1]
                    + 3.0 * t_inv * t_inv * t * p1[1]
                    + 3.0 * t_inv * t * t * p2[1]
                    + t * t * t * p[1],
            ];
            self.segments.push(GpuSegment {
                p0: self.current_point,
                p1: next_p,
            });
            self.current_point = next_p;
        }
    }
    fn close(&mut self) {
        if self.current_point != self.first_point {
            self.segments.push(GpuSegment {
                p0: self.current_point,
                p1: self.first_point,
            });
            self.current_point = self.first_point;
        }
    }
}
