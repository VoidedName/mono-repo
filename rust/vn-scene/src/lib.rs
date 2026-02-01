use std::hash::{Hash, Hasher};
use std::rc::Rc;
use vn_ui_animation_macros::Interpolatable;

fn hash_f32<H: Hasher>(state: &mut H, f: f32) {
    f.to_bits().hash(state);
}

fn hash_f32_slice<H: Hasher>(state: &mut H, slice: &[f32]) {
    for &f in slice {
        hash_f32(state, f);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Interpolatable)]
pub struct TextureId(#[interpolate_snappy = "snap_middle"] pub Rc<u32>);

impl std::fmt::Display for TextureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents an RGBA color.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable, Interpolatable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Eq for Color {}

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_f32(state, self.r);
        hash_f32(state, self.g);
        hash_f32(state, self.b);
        hash_f32(state, self.a);
    }
}

impl Color {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const MAGENTA: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TURQUOISE: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const YELLOW: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    /// A fully transparent color.
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Returns a new color with the specified opacity, adjusting RGB values for premultiplied alpha.
    pub fn with_alpha(self, opacity: f32) -> Self {
        if opacity == 0.0 || self.a == 0.0 {
            return Self::TRANSPARENT;
        }

        Self {
            r: self.r / self.a * opacity,
            g: self.g / self.a * opacity,
            b: self.b / self.a * opacity,
            a: opacity,
        }
    }

    pub fn lighten(self, amount: f32) -> Self {
        Self {
            r: (self.r + amount).min(1.0),
            g: (self.g + amount).min(1.0),
            b: (self.b + amount).min(1.0),
            a: self.a,
        }
    }

    pub fn darken(self, amount: f32) -> Self {
        Self {
            r: (self.r - amount).max(0.0),
            g: (self.g - amount).max(0.0),
            b: (self.b - amount).max(0.0),
            a: self.a,
        }
    }
}

/// A simple 2D rectangle defined by position and size.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable, Interpolatable)]
pub struct Rect {
    pub position: [f32; 2],
    pub size: [f32; 2],
}

impl Eq for Rect {}

impl Hash for Rect {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_f32_slice(state, &self.position);
        hash_f32_slice(state, &self.size);
    }
}

impl Rect {
    pub fn contains(&self, point: [f32; 2]) -> bool {
        point[0] >= self.position[0]
            && point[0] <= self.position[0] + self.size[0]
            && point[1] >= self.position[1]
            && point[1] <= self.position[1] + self.size[1]
    }

    pub fn intersect(&self, other: &Self) -> Self {
        let x1 = self.position[0].max(other.position[0]);
        let y1 = self.position[1].max(other.position[1]);
        let x2 = (self.position[0] + self.size[0]).min(other.position[0] + other.size[0]);
        let y2 = (self.position[1] + self.size[1]).min(other.position[1] + other.size[1]);

        let width = (x2 - x1).max(0.0);
        let height = (y2 - y1).max(0.0);

        let s = Self {
            position: [x1, y1],
            size: [width, height],
        };

        if !s.position[0].is_finite()
            || !s.position[1].is_finite()
            || !s.size[0].is_finite()
            || !s.size[1].is_finite()
        {
            panic!("Invalid rect: {:?}", self);
        }

        s
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.intersect(other).size != [0.0, 0.0]
    }

    pub fn union(&self, other: &Self) -> Self {
        let x1 = self.position[0].min(other.position[0]);
        let y1 = self.position[1].min(other.position[1]);
        let x2 = (self.position[0] + self.size[0]).max(other.position[0] + other.size[0]);
        let y2 = (self.position[1] + self.size[1]).max(other.position[1] + other.size[1]);

        Self {
            position: [x1, y1],
            size: [x2 - x1, y2 - y1],
        }
    }

    /// A rectangle that effectively disables clipping by covering a massive area.
    pub const NO_CLIP: Self = Self {
        position: [f32::MIN / 2.0, f32::MIN / 2.0],
        size: [f32::MAX, f32::MAX],
    };

    /// A unit rectangle i.e. size of 1
    pub const UNIT: Self = Self {
        position: [0.0, 0.0],
        size: [1.0, 1.0],
    };

    /// Creates a new builder for a [`Rect`].
    pub fn builder() -> RectBuilder {
        RectBuilder::new()
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

/// Represents a 2D transformation including translation, rotation, scale, and origin.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable, Interpolatable)]
pub struct Transform {
    pub translation: [f32; 2],
    /// Rotation in radians.
    pub rotation: f32,
    pub scale: [f32; 2],
    /// The pivot point for rotation and scaling, typically in normalized coordinates [0, 1].
    pub origin: [f32; 2],
}

impl Eq for Transform {}

impl Hash for Transform {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_f32_slice(state, &self.translation);
        hash_f32(state, self.rotation);
        hash_f32_slice(state, &self.scale);
        hash_f32_slice(state, &self.origin);
    }
}

impl Transform {
    /// Identity transform: no translation, no rotation, unit scale, origin at top left.
    pub const DEFAULT: Self = Self {
        translation: [0.0, 0.0],
        rotation: 0.0,
        scale: [1.0, 1.0],
        origin: [0.0, 0.0],
    };

    pub fn builder() -> TransformBuilder {
        TransformBuilder::new()
    }
}

/// A builder for creating [`Transform`] instances.
pub struct TransformBuilder {
    transform: Transform,
}

impl TransformBuilder {
    pub fn new() -> Self {
        Self {
            transform: Transform::DEFAULT,
        }
    }

    pub fn translation(mut self, translation: [f32; 2]) -> Self {
        self.transform.translation = translation;
        self
    }

    pub fn rotation(mut self, rotation: f32) -> Self {
        self.transform.rotation = rotation;
        self
    }

    pub fn scale(mut self, scale: [f32; 2]) -> Self {
        self.transform.scale = scale;
        self
    }

    pub fn origin(mut self, origin: [f32; 2]) -> Self {
        self.transform.origin = origin;
        self
    }

    pub fn build(self) -> Transform {
        self.transform
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub resolution: [f32; 2],
}

pub use winit::event::{ElementState, KeyEvent};
pub use winit::keyboard::{KeyCode, PhysicalKey};

pub trait Scene {
    fn add_box(&mut self, b: BoxPrimitiveData);
    fn add_image(&mut self, i: ImagePrimitiveData);
    fn add_text(&mut self, t: TextPrimitiveData);
    fn with_next_layer(&mut self, f: &mut dyn FnMut(&mut dyn Scene));
    fn with_top_layer(&mut self, f: &mut dyn FnMut(&mut dyn Scene));
    fn current_layer_id(&self) -> u32;
    fn layers(&self) -> &[Layer];
    fn extend(&mut self, other: &mut dyn Scene);
}

/// A collection of primitives to be rendered together.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Layer {
    pub boxes: Vec<BoxPrimitiveData>,
    pub images: Vec<ImagePrimitiveData>,
    pub texts: Vec<TextPrimitiveData>,
}

impl Layer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_box(&mut self, b: BoxPrimitiveData) {
        self.boxes.push(b);
    }

    pub fn add_image(&mut self, i: ImagePrimitiveData) {
        self.images.push(i);
    }

    pub fn add_text(&mut self, t: TextPrimitiveData) {
        self.texts.push(t);
    }
}

// These are data-only versions of primitives to be used in the trait
#[derive(Debug, Clone, PartialEq)]
pub struct BoxPrimitiveData {
    pub transform: Transform,
    pub size: [f32; 2],
    pub color: Color,
    pub border_color: Color,
    pub border_thickness: f32,
    pub border_radius: f32,
    pub clip_rect: Rect,
}

impl Eq for BoxPrimitiveData {}

impl Hash for BoxPrimitiveData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.transform.hash(state);
        hash_f32_slice(state, &self.size);
        self.color.hash(state);
        self.border_color.hash(state);
        hash_f32(state, self.border_thickness);
        hash_f32(state, self.border_radius);
        self.clip_rect.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImagePrimitiveData {
    pub transform: Transform,
    /// Render Size
    pub size: [f32; 2],
    pub tint: Color,
    pub texture_id: TextureId,
    /// This will clip the rendered image to the clip_rect (if clip rect does not cover the entire size)
    pub clip_rect: Rect,
    /// Area of the texture to render in NDC.
    pub uv_rect: Rect,
}

impl Eq for ImagePrimitiveData {}

impl Hash for ImagePrimitiveData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.transform.hash(state);
        hash_f32_slice(state, &self.size);
        self.tint.hash(state);
        self.texture_id.hash(state);
        self.clip_rect.hash(state);
        self.uv_rect.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextPrimitiveData {
    pub transform: Transform,
    pub tint: Color,
    pub glyphs: Vec<GlyphInstanceData>,
    pub clip_rect: Rect,
}

impl Eq for TextPrimitiveData {}

impl Hash for TextPrimitiveData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.transform.hash(state);
        self.tint.hash(state);
        self.glyphs.hash(state);
        self.clip_rect.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphInstanceData {
    pub texture_id: TextureId,
    pub position: [f32; 2],
    pub size: [f32; 2],
    /// NDC coordinates.
    pub uv_rect: Rect,
}

impl Eq for GlyphInstanceData {}

impl Hash for GlyphInstanceData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.texture_id.hash(state);
        hash_f32_slice(state, &self.position);
        hash_f32_slice(state, &self.size);
        self.uv_rect.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct GlyphData {
    pub texture_id: TextureId,
    pub advance: f32,
    pub x_bearing: f32,
    pub y_offset: f32,
    pub size: [f32; 2],
    /// NDC coordinates.
    pub uv_rect: Rect,
}
