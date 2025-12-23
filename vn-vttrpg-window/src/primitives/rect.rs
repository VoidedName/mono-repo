use crate::graphics::VertexDescription;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn full_screen() -> Self {
        Self { x: -1.0, y: -1.0, width: 2.0, height: 2.0 }
    }
}

impl VertexDescription for Rect {
    fn stride() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
    }

    fn location_count() -> u32 {
        1
    }

    fn size_in_buffer() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
    }

    fn attributes(
        shader_location_start: u32,
        offset: wgpu::BufferAddress,
    ) -> Vec<wgpu::VertexAttribute> {
        vec![wgpu::VertexAttribute {
            offset,
            shader_location: shader_location_start,
            format: wgpu::VertexFormat::Float32x4,
        }]
    }
}
