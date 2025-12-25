use crate::graphics::VertexDescription;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Rect {
    pub position: [f32; 2],
    pub size: [f32; 2],
}

pub struct RectBuilder {
    rect: Rect,
}

impl RectBuilder {
    pub fn new() -> Self {
        Self {
            rect: Rect::NO_CLIP,
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

impl Rect {
    // might be worth thinking about "flags" to send to the shader to enable clipping
    // instead of defining a huge clip area... realistically, we should never exceed these limits
    // as f32s completely break down in precision here anyway and anything rendered at such huge
    // translations would be completely incomprehensible due to imprecision anyway.
    pub const NO_CLIP: Self = Self {
        position: [f32::MIN / 2.0, f32::MIN / 2.0],
        size: [f32::MAX, f32::MAX],
    };

    pub fn builder() -> RectBuilder {
        RectBuilder::new()
    }
}

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
