use crate::graphics::VertexDescription;
use crate::primitives::color::Color;
use crate::primitives::properties::PrimitiveProperties;
use crate::texture::TextureId;

/// Internal representation of a textured primitive sent to the GPU.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct _TexturePrimitive {
    /// Common properties shared by all primitives (transform, clipping).
    pub common: PrimitiveProperties,
    // Remark (coordinates): should this be in NDC or Pixels?
    /// NDC coordinates.
    pub uv_rect: vn_scene::Rect,
    pub size: [f32; 2],
    pub tint: Color,
}

/// A builder for creating [`_TexturePrimitive`] instances.
pub struct TexturePrimitiveBuilder {
    primitive: _TexturePrimitive,
}

impl TexturePrimitiveBuilder {
    pub fn new() -> Self {
        Self {
            primitive: _TexturePrimitive {
                common: PrimitiveProperties::DEFAULT,
                uv_rect: vn_scene::Rect::NO_CLIP,
                size: [1.0, 1.0],
                tint: Color::WHITE,
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
        PrimitiveProperties::location_count() + 1 + 1 + Color::location_count()
    }

    fn attributes(
        shader_location_start: u32,
        offset: wgpu::BufferAddress,
    ) -> Vec<wgpu::VertexAttribute> {
        let mut attrs = PrimitiveProperties::attributes(shader_location_start, offset);
        let mut current_location = shader_location_start + PrimitiveProperties::location_count();
        let mut current_offset = offset + PrimitiveProperties::stride();

        // uv_rect
        attrs.push(wgpu::VertexAttribute {
            offset: current_offset,
            shader_location: current_location,
            format: wgpu::VertexFormat::Float32x4,
        });
        current_location += 1;
        current_offset += size_of::<vn_scene::Rect>() as wgpu::BufferAddress;

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

/// A primitive for rendering images with a texture.
#[derive(Debug, Clone)]
pub struct ImagePrimitive {
    /// Common properties shared by all primitives (transform, clipping).
    pub common: PrimitiveProperties,
    pub size: [f32; 2],
    /// NDC coordinates.
    pub uv_rect: vn_scene::Rect,
    pub texture: TextureId,
    pub tint: Color,
}

/// A builder for creating [`ImagePrimitive`] instances.
pub struct ImagePrimitiveBuilder {
    primitive: ImagePrimitive,
}

impl ImagePrimitiveBuilder {
    /// Creates a new builder for an [`ImagePrimitive`] with the specified texture.
    pub fn new(texture: TextureId) -> Self {
        Self {
            primitive: ImagePrimitive {
                common: PrimitiveProperties::DEFAULT,
                size: [1.0, 1.0],
                uv_rect: vn_scene::Rect::builder().build(),
                texture,
                tint: Color::WHITE,
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

    pub fn tint(mut self, tint: Color) -> Self {
        self.primitive.tint = tint;
        self
    }

    pub fn uv_rect<F>(mut self, f: F) -> Self
    where
        F: FnOnce(vn_scene::RectBuilder) -> vn_scene::RectBuilder,
    {
        self.primitive.uv_rect = f(vn_scene::Rect::builder()).build();
        self
    }

    pub fn build(self) -> ImagePrimitive {
        self.primitive
    }
}

impl ImagePrimitive {
    pub fn builder(texture: TextureId) -> ImagePrimitiveBuilder {
        ImagePrimitiveBuilder::new(texture)
    }
}

/// A primitive for rendering text with a specific font and size.
#[derive(Debug, Clone)]
pub struct TextPrimitive {
    /// Common properties shared by all primitives (transform, clipping).
    pub common: PrimitiveProperties,
    pub glyphs: Vec<GlyphInstance>,
    pub tint: Color,
}

#[derive(Debug, Clone)]
pub struct GlyphInstance {
    pub texture: TextureId,
    pub position: [f32; 2],
    pub size: [f32; 2],
    /// NDC coordinates.
    pub uv_rect: vn_scene::Rect,
}

/// A builder for creating [`TextPrimitive`] instances.
pub struct TextPrimitiveBuilder {
    primitive: TextPrimitive,
}

impl TextPrimitiveBuilder {
    /// Creates a new builder for a [`TextPrimitive`] with the specified text and font.
    pub fn new() -> Self {
        Self {
            primitive: TextPrimitive {
                common: PrimitiveProperties::DEFAULT,
                glyphs: Vec::new(),
                tint: Color::WHITE,
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

    pub fn add_glyph(mut self, glyph: GlyphInstance) -> Self {
        self.primitive.glyphs.push(glyph);
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
    pub fn builder() -> TextPrimitiveBuilder {
        TextPrimitiveBuilder::new()
    }

    pub fn to_texture_primitive(&self) -> _TexturePrimitive {
        _TexturePrimitive {
            common: self.common,
            uv_rect: vn_scene::Rect::NO_CLIP,
            size: [0.0, 0.0],
            tint: self.tint,
        }
    }
}

impl ImagePrimitive {
    pub fn to_texture_primitive(&self) -> _TexturePrimitive {
        _TexturePrimitive {
            common: self.common,
            uv_rect: self.uv_rect,
            size: self.size,
            tint: self.tint,
        }
    }
}
