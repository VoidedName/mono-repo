use crate::graphics::VertexDescription;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Transform {
    pub translation: [f32; 2],
    pub rotation: f32, // In radians
    pub scale: [f32; 2],
    pub origin: [f32; 2],
}

impl Transform {
    pub const DEFAULT: Self = Self {
        translation: [0.0, 0.0],
        rotation: 0.0,
        scale: [1.0, 1.0],
        origin: [0.5, 0.5],
    };
}

pub struct TransformBuilder {
    transform: Transform,
}

impl TransformBuilder {
    pub fn new() -> Self {
        Self {
            transform: Transform::DEFAULT,
        }
    }

    pub fn translation(mut self, translation: [f32; 2]) -> Self {
        self.transform.translation = translation;
        self
    }

    pub fn rotation(mut self, rotation: f32) -> Self {
        self.transform.rotation = rotation;
        self
    }

    pub fn scale(mut self, scale: [f32; 2]) -> Self {
        self.transform.scale = scale;
        self
    }

    pub fn origin(mut self, origin: [f32; 2]) -> Self {
        self.transform.origin = origin;
        self
    }

    pub fn build(self) -> Transform {
        self.transform
    }
}

impl Transform {
    pub fn builder() -> TransformBuilder {
        TransformBuilder::new()
    }
}

impl VertexDescription for Transform {
    fn location_count() -> u32 {
        4
    }

    fn attributes(
        shader_location_start: u32,
        offset: wgpu::BufferAddress,
    ) -> Vec<wgpu::VertexAttribute> {
        vec![
            wgpu::VertexAttribute {
                offset,
                shader_location: shader_location_start,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: offset + size_of::<[f32; 2]>() as wgpu::BufferAddress,
                shader_location: shader_location_start + 1,
                format: wgpu::VertexFormat::Float32,
            },
            wgpu::VertexAttribute {
                offset: offset + (size_of::<[f32; 2]>() + size_of::<f32>()) as wgpu::BufferAddress,
                shader_location: shader_location_start + 2,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: offset
                    + (size_of::<[f32; 2]>() * 2 + size_of::<f32>()) as wgpu::BufferAddress,
                shader_location: shader_location_start + 3,
                format: wgpu::VertexFormat::Float32x2,
            },
        ]
    }
}
