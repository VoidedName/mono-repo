use crate::graphics::VertexDescription;
use crate::primitives::rect::Rect;
use crate::primitives::transform::Transform;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PrimitiveProperties {
    pub transform: Transform,
    pub clip_area: Rect,
}

impl PrimitiveProperties {
    pub const DEFAULT: Self = Self {
        transform: Transform::DEFAULT,
        clip_area: Rect::NO_CLIP,
    };
}

pub struct PrimitivePropertiesBuilder {
    properties: PrimitiveProperties,
}

impl PrimitivePropertiesBuilder {
    pub fn new() -> Self {
        Self {
            properties: PrimitiveProperties::DEFAULT,
        }
    }

    pub fn transform(mut self, transform: Transform) -> Self {
        self.properties.transform = transform;
        self
    }

    pub fn clip_area(mut self, clip_area: Rect) -> Self {
        self.properties.clip_area = clip_area;
        self
    }

    pub fn build(self) -> PrimitiveProperties {
        self.properties
    }
}

impl PrimitiveProperties {
    pub fn builder() -> PrimitivePropertiesBuilder {
        PrimitivePropertiesBuilder::new()
    }
}

impl VertexDescription for PrimitiveProperties {
    fn location_count() -> u32 {
        Transform::location_count() + Rect::location_count()
    }

    fn attributes(
        shader_location_start: u32,
        offset: wgpu::BufferAddress,
    ) -> Vec<wgpu::VertexAttribute> {
        let mut attrs = Transform::attributes(shader_location_start, offset);
        attrs.extend(Rect::attributes(
            shader_location_start + Transform::location_count(),
            offset + Transform::stride(),
        ));
        attrs
    }
}
