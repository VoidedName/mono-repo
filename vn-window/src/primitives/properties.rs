use crate::graphics::VertexDescription;
use crate::primitives::rect::Rect;
use crate::primitives::transform::Transform;

/// Common properties shared by all rendering primitives.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PrimitiveProperties {
    pub transform: Transform,
    /// The rectangular area where the primitive is visible.
    pub clip_area: Rect,
}

impl PrimitiveProperties {
    /// The default set of properties: identity transform and no clipping.
    pub const DEFAULT: Self = Self {
        transform: Transform::DEFAULT,
        clip_area: Rect::NO_CLIP,
    };
}

/// A builder for creating [`PrimitiveProperties`] instances.
pub struct PrimitivePropertiesBuilder {
    properties: PrimitiveProperties,
}

impl PrimitivePropertiesBuilder {
    pub fn new() -> Self {
        Self {
            properties: PrimitiveProperties::DEFAULT,
        }
    }

    pub fn transform<F>(mut self, f: F) -> Self
    where
        F: FnOnce(vn_scene::TransformBuilder) -> vn_scene::TransformBuilder,
    {
        self.properties.transform = f(Transform::builder()).build();
        self
    }

    pub fn clip_area<F>(mut self, f: F) -> Self
    where
        F: FnOnce(vn_scene::RectBuilder) -> vn_scene::RectBuilder,
    {
        self.properties.clip_area = f(Rect::builder()).build();
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
