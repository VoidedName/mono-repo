use crate::TextureDescriptor;
use crate::graphics::VertexDescription;
use crate::primitives::color::Color;
use crate::primitives::properties::PrimitiveProperties;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct _TexturePrimitive {
    pub common: PrimitiveProperties,
    pub size: [f32; 2],
    pub tint: Color,
}

pub struct TexturePrimitiveBuilder {
    primitive: _TexturePrimitive,
}

impl TexturePrimitiveBuilder {
    pub fn new() -> Self {
        Self {
            primitive: _TexturePrimitive {
                common: PrimitiveProperties::DEFAULT,
                size: [1.0, 1.0],
                tint: Color::WHITE,
            },
        }
    }

    pub fn common(mut self, common: PrimitiveProperties) -> Self {
        self.primitive.common = common;
        self
    }

    pub fn size(mut self, size: [f32; 2]) -> Self {
        self.primitive.size = size;
        self
    }

    pub fn tint(mut self, tint: Color) -> Self {
        self.primitive.tint = tint;
        self
    }

    pub fn build(self) -> _TexturePrimitive {
        self.primitive
    }
}

impl _TexturePrimitive {
    pub fn builder() -> TexturePrimitiveBuilder {
        TexturePrimitiveBuilder::new()
    }
}

impl VertexDescription for _TexturePrimitive {
    fn location_count() -> u32 {
        PrimitiveProperties::location_count() + 1 + Color::location_count()
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

pub struct ImagePrimitiveBuilder {
    primitive: ImagePrimitive,
}

impl ImagePrimitiveBuilder {
    pub fn new(texture: TextureDescriptor) -> Self {
        Self {
            primitive: ImagePrimitive {
                common: PrimitiveProperties::DEFAULT,
                size: [1.0, 1.0],
                texture,
                tint: Color::WHITE,
            },
        }
    }

    pub fn common(mut self, common: PrimitiveProperties) -> Self {
        self.primitive.common = common;
        self
    }

    pub fn size(mut self, size: [f32; 2]) -> Self {
        self.primitive.size = size;
        self
    }

    pub fn tint(mut self, tint: Color) -> Self {
        self.primitive.tint = tint;
        self
    }

    pub fn build(self) -> ImagePrimitive {
        self.primitive
    }
}

impl ImagePrimitive {
    pub fn builder(texture: TextureDescriptor) -> ImagePrimitiveBuilder {
        ImagePrimitiveBuilder::new(texture)
    }
}

#[derive(Debug, Clone)]
pub struct TextPrimitive {
    pub common: PrimitiveProperties,
    pub size: [f32; 2],
    pub text: String,
    pub font: String, // font name in resource manager
    pub font_size: f32,
    pub tint: Color,
}

pub struct TextPrimitiveBuilder {
    primitive: TextPrimitive,
}

impl TextPrimitiveBuilder {
    pub fn new(text: String, font: String) -> Self {
        Self {
            primitive: TextPrimitive {
                common: PrimitiveProperties::DEFAULT,
                size: [1.0, 1.0],
                text,
                font,
                font_size: 16.0,
                tint: Color::WHITE,
            },
        }
    }

    pub fn common(mut self, common: PrimitiveProperties) -> Self {
        self.primitive.common = common;
        self
    }

    pub fn size(mut self, size: [f32; 2]) -> Self {
        self.primitive.size = size;
        self
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.primitive.font_size = font_size;
        self
    }

    pub fn tint(mut self, tint: Color) -> Self {
        self.primitive.tint = tint;
        self
    }

    pub fn build(self) -> TextPrimitive {
        self.primitive
    }
}

impl TextPrimitive {
    pub fn builder(text: String, font: String) -> TextPrimitiveBuilder {
        TextPrimitiveBuilder::new(text, font)
    }

    pub fn to_texture_primitive(&self) -> _TexturePrimitive {
        _TexturePrimitive {
            common: self.common,
            size: self.size,
            tint: self.tint,
        }
    }
}

impl ImagePrimitive {
    pub fn to_texture_primitive(&self) -> _TexturePrimitive {
        _TexturePrimitive {
            common: self.common,
            size: self.size,
            tint: self.tint,
        }
    }
}
