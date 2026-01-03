use crate::graphics::VertexDescription;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
}

impl VertexDescription for Vertex {
    fn location_count() -> u32 {
        <[f32; 2]>::location_count()
    }

    fn attributes(
        shader_location_start: u32,
        offset: wgpu::BufferAddress,
    ) -> Vec<wgpu::VertexAttribute> {
        <[f32; 2]>::attributes(shader_location_start, offset)
    }
}
