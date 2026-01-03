use crate::graphics::VertexDescription;
pub use vn_scene::Rect;

impl VertexDescription for Rect {
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
            format: wgpu::VertexFormat::Float32x4,
        }]
    }
}

pub struct RectBuilder {
    rect: Rect,
}

impl RectBuilder {
    pub fn new() -> Self {
        Self {
            rect: Rect {
                position: [0.0, 0.0],
                size: [0.0, 0.0],
            },
        }
    }

    pub fn position(mut self, position: [f32; 2]) -> Self {
        self.rect.position = position;
        self
    }

    pub fn size(mut self, size: [f32; 2]) -> Self {
        self.rect.size = size;
        self
    }

    pub fn build(self) -> Rect {
        self.rect
    }
}
