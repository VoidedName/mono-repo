use crate::graphics::VertexDescription;
use crate::TextureDescriptor;
use crate::primitives::color::Color;
use crate::primitives::properties::PrimitiveProperties;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexturePrimitive {
    pub common: PrimitiveProperties,
    pub size: [f32; 2],
    pub tint: Color,
}

impl VertexDescription for TexturePrimitive {
    fn stride() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
    }

    fn location_count() -> u32 {
        PrimitiveProperties::location_count() + 1 + Color::location_count()
    }

    fn size_in_buffer() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
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

        attrs.extend(Color::attributes(current_location, current_offset));
        attrs
    }
}

#[derive(Debug, Clone)]
pub struct ImagePrimitive {
    pub common: PrimitiveProperties,
    pub size: [f32; 2],
    pub texture: TextureDescriptor,
    pub tint: Color,
}

impl ImagePrimitive {
    pub fn to_texture_primitive(&self) -> TexturePrimitive {
        TexturePrimitive {
            common: self.common,
            size: self.size,
            tint: self.tint,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextPrimitive {
    pub common: PrimitiveProperties,
    pub size: [f32; 2],
    // For now, text is expected to be rendered to a texture (e.g. SVG or glyph cache)
    pub texture: TextureDescriptor,
    pub tint: Color,
}

impl TextPrimitive {
    pub fn to_texture_primitive(&self) -> TexturePrimitive {
        TexturePrimitive {
            common: self.common,
            size: self.size,
            tint: self.tint,
        }
    }
}
