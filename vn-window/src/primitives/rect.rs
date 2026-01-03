use crate::graphics::VertexDescription;
use vn_ui_animation_macros::Interpolatable;

/// A simple 2D rectangle defined by position and size.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Interpolatable)]
pub struct Rect {
    pub position: [f32; 2],
    pub size: [f32; 2],
}

impl Rect {
    pub fn contains(&self, point: [f32; 2]) -> bool {
        point[0] >= self.position[0]
            && point[0] <= self.position[0] + self.size[0]
            && point[1] >= self.position[1]
            && point[1] <= self.position[1] + self.size[1]
    }
}

/// A builder for creating [`Rect`] instances.
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

impl Rect {
    /// A rectangle that effectively disables clipping by covering a massive area.
    pub const NO_CLIP: Self = Self {
        position: [f32::MIN / 2.0, f32::MIN / 2.0],
        size: [f32::MAX, f32::MAX],
    };

    /// Creates a new builder for a [`Rect`].
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
