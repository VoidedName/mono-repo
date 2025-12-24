use crate::errors::RenderError;
use crate::graphics::VertexLayout;

pub struct PipelineBuilder<'a> {
    device: &'a wgpu::Device,
    label: Option<&'a str>,
    shader: Option<&'a wgpu::ShaderModule>,
    vertex_layouts: Vec<VertexLayout>,
    bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
    color_format: wgpu::TextureFormat,
    primitive: wgpu::PrimitiveState,
    depth_stencil: Option<wgpu::DepthStencilState>,
    multisample: wgpu::MultisampleState,
    blend: Option<wgpu::BlendState>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(device: &'a wgpu::Device, color_format: wgpu::TextureFormat) -> Self {
        Self {
            device,
            label: None,
            shader: None,
            vertex_layouts: Vec::new(),
            bind_group_layouts: Vec::new(),
            color_format,
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
            blend: None,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn shader(mut self, shader: &'a wgpu::ShaderModule) -> Self {
        self.shader = Some(shader);
        self
    }

    pub fn blend(mut self, blend: wgpu::BlendState) -> Self {
        self.blend = Some(blend);
        self
    }

    pub fn add_vertex_layout(mut self, layout: VertexLayout) -> Self {
        self.vertex_layouts.push(layout);
        self
    }

    pub fn add_bind_group_layout(mut self, layout: &'a wgpu::BindGroupLayout) -> Self {
        self.bind_group_layouts.push(layout);
        self
    }

    pub fn build(self) -> Result<wgpu::RenderPipeline, RenderError> {
        let shader = self.shader.ok_or_else(|| RenderError::PipelineError("Shader not set".to_string()))?;

        let layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: self.label,
            bind_group_layouts: &self.bind_group_layouts,
            immediate_size: 0,
        });

        let vertex_buffers: Vec<wgpu::VertexBufferLayout> = self.vertex_layouts.iter().map(|l| {
            wgpu::VertexBufferLayout {
                array_stride: l.array_stride,
                step_mode: l.step_mode,
                attributes: &l.attributes,
            }
        }).collect();

        let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: self.label,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.color_format,
                    blend: self.blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            multiview_mask: None,
            cache: None,
        });

        Ok(pipeline)
    }
}
