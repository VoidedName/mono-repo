use std::sync::Arc;
use crate::graphics::VertexDescription;
use crate::Texture;
use crate::primitives::color::Color;
use crate::primitives::properties::PrimitiveProperties;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexturePrimitive {
    pub common: PrimitiveProperties,
    pub tint: Color,
}

impl VertexDescription for TexturePrimitive {
    fn stride() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
    }

    fn location_count() -> u32 {
        PrimitiveProperties::location_count() + Color::location_count()
    }

    fn size_in_buffer() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
    }

    fn attributes(
        shader_location_start: u32,
        offset: wgpu::BufferAddress,
    ) -> Vec<wgpu::VertexAttribute> {
        let mut attrs = PrimitiveProperties::attributes(shader_location_start, offset);
        let current_location = shader_location_start + PrimitiveProperties::location_count();
        let current_offset = offset + PrimitiveProperties::stride();

        attrs.extend(Color::attributes(current_location, current_offset));
        attrs
    }
}

#[derive(Debug, Clone)]
pub struct ImagePrimitive {
    pub common: PrimitiveProperties,
    pub texture: Arc<Texture>,
    pub tint: Color,
}

impl ImagePrimitive {
    pub fn to_texture_primitive(&self) -> TexturePrimitive {
        TexturePrimitive {
            common: self.common,
            tint: self.tint,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextPrimitive {
    pub common: PrimitiveProperties,
    // For now, text is expected to be rendered to a texture (e.g. SVG or glyph cache)
    pub texture: Arc<Texture>,
    pub tint: Color,
}

impl TextPrimitive {
    pub fn to_texture_primitive(&self) -> TexturePrimitive {
        TexturePrimitive {
            common: self.common,
            tint: self.tint,
        }
    }
}
