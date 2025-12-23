use crate::graphics::VertexDescription;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Rect {
    pub position: [f32; 2],
    pub size: [f32; 2],
}

impl Default for Rect {
    fn default() -> Self {
        Self::NO_CLIP
    }
}

impl Rect {
    // might be worth thinking about "flags" to send to the shader to enable clipping
    // instead of defining a huge clip area... realistically, we should never exceed these limits
    // as f32s completely break down in precision here anyway and anything rendered at such huge
    // translations would be completely incomprehensible due to imprecision anyway.
    pub const NO_CLIP: Self = Self {
        position: [f32::MIN / 2.0, f32::MIN / 2.0],
        size: [f32::MAX, f32::MAX],
    };
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
