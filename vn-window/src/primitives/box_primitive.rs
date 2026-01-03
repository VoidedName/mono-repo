use crate::graphics::VertexDescription;
use crate::primitives::color::Color;
use crate::primitives::properties::PrimitiveProperties;

/// A rendering primitive representing a rectangle with an optional border and rounded corners.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BoxPrimitive {
    /// Common properties shared by all primitives (transform, clipping).
    pub common: PrimitiveProperties,
    pub size: [f32; 2],
    pub color: Color,
    pub border_color: Color,
    pub border_thickness: f32,
    pub corner_radius: f32,
}

/// A builder for creating [`BoxPrimitive`] instances with a fluent API.
pub struct BoxPrimitiveBuilder {
    primitive: BoxPrimitive,
}

impl BoxPrimitiveBuilder {
    pub fn new() -> Self {
        Self {
            primitive: BoxPrimitive {
                common: PrimitiveProperties::DEFAULT,
                size: [1.0, 1.0],
                color: Color::TRANSPARENT,
                border_color: Color::TRANSPARENT,
                border_thickness: 0.0,
                corner_radius: 0.0,
            },
        }
    }

    pub fn common(mut self, common: PrimitiveProperties) -> Self {
        self.primitive.common = common;
        self
    }

    //noinspection ALL (duplicate code)
    pub fn transform<F>(mut self, f: F) -> Self
    where
        F: FnOnce(vn_scene::TransformBuilder) -> vn_scene::TransformBuilder,
    {
        self.primitive.common.transform = f(vn_scene::Transform::builder()).build();
        self
    }

    //noinspection ALL (duplicate code)
    pub fn clip_area<F>(mut self, f: F) -> Self
    where
        F: FnOnce(vn_scene::RectBuilder) -> vn_scene::RectBuilder,
    {
        self.primitive.common.clip_area = f(vn_scene::Rect::builder()).build();
        self
    }

    pub fn size(mut self, size: [f32; 2]) -> Self {
        self.primitive.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.primitive.color = color;
        self
    }

    pub fn border_color(mut self, border_color: Color) -> Self {
        self.primitive.border_color = border_color;
        self
    }

    pub fn border_thickness(mut self, border_thickness: f32) -> Self {
        self.primitive.border_thickness = border_thickness;
        self
    }

    pub fn corner_radius(mut self, corner_radius: f32) -> Self {
        self.primitive.corner_radius = corner_radius;
        self
    }

    pub fn build(self) -> BoxPrimitive {
        self.primitive
    }
}

impl BoxPrimitive {
    /// Creates a new builder for a [`BoxPrimitive`].
    pub fn builder() -> BoxPrimitiveBuilder {
        BoxPrimitiveBuilder::new()
    }
}

impl VertexDescription for BoxPrimitive {
    fn location_count() -> u32 {
        PrimitiveProperties::location_count() + 1 + Color::location_count() * 2 + 2 // size (1) + color (1) + border_color (1) + thickness (1) + radius (1) = 5 locations
    }

    fn attributes(
        shader_location_start: u32,
        offset: wgpu::BufferAddress,
    ) -> Vec<wgpu::VertexAttribute> {
        let mut attrs = PrimitiveProperties::attributes(shader_location_start, offset);
        let mut current_location = shader_location_start + PrimitiveProperties::location_count();
        let mut current_offset = offset + PrimitiveProperties::stride();

        // size
        attrs.push(wgpu::VertexAttribute {
            offset: current_offset,
            shader_location: current_location,
            format: wgpu::VertexFormat::Float32x2,
        });
        current_location += 1;
        current_offset += size_of::<[f32; 2]>() as wgpu::BufferAddress;

        // color
        attrs.extend(Color::attributes(current_location, current_offset));
        current_location += Color::location_count();
        current_offset += Color::stride();

        // border_color
        attrs.extend(Color::attributes(current_location, current_offset));
        current_location += Color::location_count();
        current_offset += Color::stride();

        // border_thickness (Float32)
        attrs.push(wgpu::VertexAttribute {
            offset: current_offset,
            shader_location: current_location,
            format: wgpu::VertexFormat::Float32,
        });
        current_location += 1;
        current_offset += size_of::<f32>() as wgpu::BufferAddress;

        // corner_radius (Float32)
        attrs.push(wgpu::VertexAttribute {
            offset: current_offset,
            shader_location: current_location,
            format: wgpu::VertexFormat::Float32,
        });

        attrs
    }
}
