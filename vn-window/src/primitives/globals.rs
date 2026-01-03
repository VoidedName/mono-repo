use crate::graphics::VertexDescription;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub resolution: [f32; 2],
}

impl VertexDescription for Globals {
    fn location_count() -> u32 {
        1
    }

    fn attributes(
        shader_location_start: u32,
        offset: wgpu::BufferAddress,
    ) -> Vec<wgpu::VertexAttribute> {
        vec![wgpu::VertexAttribute {
            offset,
            shader_location: shader_location_start,
            format: wgpu::VertexFormat::Float32x2,
        }]
    }
}
