use crate::graphics::VertexDescription;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Transform {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32, // In radians
    pub scale: [f32; 2],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            size: [1.0, 1.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        }
    }
}

impl VertexDescription for Transform {
    fn stride() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
    }

    fn location_count() -> u32 {
        4
    }

    fn size_in_buffer() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
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
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: offset + size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: shader_location_start + 2,
                format: wgpu::VertexFormat::Float32,
            },
            wgpu::VertexAttribute {
                offset: offset + size_of::<[f32; 5]>() as wgpu::BufferAddress,
                shader_location: shader_location_start + 3,
                format: wgpu::VertexFormat::Float32x2,
            },
        ]
    }
}
