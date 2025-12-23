use crate::graphics::VertexDescription;

pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], text_coords: [0.4131759, 1.0 - 0.99240386], },
    Vertex { position: [-0.49513406, 0.06958647, 0.0], text_coords: [0.0048659444, 1.0 - 0.56958647], },
    Vertex { position: [-0.21918549, -0.44939706, 0.0], text_coords: [0.28081453, 1.0 - 0.05060294], },
    Vertex { position: [0.35966998, -0.3473291, 0.0], text_coords: [0.85967, 1.0 - 0.1526709], },
    Vertex { position: [0.44147372, 0.2347359, 0.0], text_coords: [0.9414737, 1.0 - 0.7347359], },
];

pub const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    text_coords: [f32; 2],
}

impl VertexDescription for Vertex {
    fn stride() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
    }

    fn location_count() -> u32 {
        2
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
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: offset + size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: shader_location_start + 1,
                format: wgpu::VertexFormat::Float32x2,
            },
        ]
    }
}