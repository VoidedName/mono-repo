use std::rc::Rc;
use vn_ui_animation_macros::Interpolatable;

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

    pub fn intersect(&self, other: &Self) -> Self {
        let x1 = self.position[0].max(other.position[0]);
        let y1 = self.position[1].max(other.position[1]);
        let x2 = (self.position[0] + self.size[0]).min(other.position[0] + other.size[0]);
        let y2 = (self.position[1] + self.size[1]).min(other.position[1] + other.size[1]);

        let width = (x2 - x1).max(0.0);
        let height = (y2 - y1).max(0.0);

        Self {
            position: [x1, y1],
            size: [width, height],
        }
    }

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
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Interpolatable)]
pub struct Transform {
    pub translation: [f32; 2],
    /// Rotation in radians.
    pub rotation: f32,
    pub scale: [f32; 2],
    /// The pivot point for rotation and scaling, typically in normalized coordinates [0, 1].
    pub origin: [f32; 2],
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
    fn current_layer_id(&self) -> u32;
}

// These are data-only versions of primitives to be used in the trait
#[derive(Debug, Clone)]
pub struct BoxPrimitiveData {
    pub transform: Transform,
    pub size: [f32; 2],
    pub color: Color,
    pub border_color: Color,
    pub border_thickness: f32,
    pub border_radius: f32,
    pub clip_rect: Rect,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct TextPrimitiveData {
    pub transform: Transform,
    pub tint: Color,
    pub glyphs: Vec<GlyphInstanceData>,
    pub clip_rect: Rect,
}

#[derive(Debug, Clone)]
pub struct GlyphInstanceData {
    pub texture_id: TextureId,
    pub position: [f32; 2],
    pub size: [f32; 2],
    /// NDC coordinates.
    pub uv_rect: Rect,
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
