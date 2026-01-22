// instead? in addition to? anyway, consider
// returning a complex size for elements instead
// usecase: while we can indicate to greedy growing components that the container is unsized
//          we can not know if the child is greedy.

use vn_ui_animation_macros::Interpolatable;

/// A concrete size with a fixed width and height.
#[derive(Debug, Clone, Copy, PartialEq, Interpolatable)]
pub struct ElementSize {
    pub width: f32,
    pub height: f32,
}

impl ElementSize {
    /// Computes the bounding box size for the rotated bounding box of the element
    pub fn rotate(&self, angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            width: self.width * cos.abs() + self.height * sin.abs(),
            height: self.width * sin.abs() + self.height * cos.abs(),
        }
    }

    pub fn shrink_by(&self, size: ElementSize) -> Self {
        Self {
            width: self.width - size.width.min(self.width),
            height: self.height - size.height.min(self.height),
        }
    }

    pub fn grow_by(&self, size: ElementSize) -> Self {
        Self {
            width: self.width + size.width,
            height: self.height + size.height,
        }
    }
}

pub type SceneSize = (f32, f32);

impl ElementSize {
    pub const ZERO: ElementSize = ElementSize {
        width: 0.0,
        height: 0.0,
    };

    pub fn clamp_to_constraints(self, constraints: SizeConstraints) -> ElementSize {
        let max_size = constraints.max_size.to_concrete();

        ElementSize {
            width: self
                .width
                .min(max_size.width)
                .max(constraints.min_size.width),
            height: self
                .height
                .min(max_size.height)
                .max(constraints.min_size.height),
        }
    }
}

/// Sometimes we need to denote that an element in its layout does not have any constraint
/// along some axis. This will be represented with a None in that axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DynamicSize {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl DynamicSize {
    pub fn to_concrete(self) -> ElementSize {
        ElementSize {
            width: self.width.unwrap_or(f32::INFINITY),
            height: self.height.unwrap_or(f32::INFINITY),
        }
    }

    pub fn shrink_by(&self, size: ElementSize) -> Self {
        Self {
            width: self.width.map(|width| width - size.width.min(width)),
            height: self.height.map(|height| height - size.height.min(height)),
        }
    }

    pub fn grow_by(&self, size: ElementSize) -> Self {
        Self {
            width: self.width.map(|width| width + size.width),
            height: self.height.map(|height| height + size.height),
        }
    }
}

/// Defines the minimum and maximum size constraints for layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeConstraints {
    pub min_size: ElementSize,
    pub max_size: DynamicSize,
    pub scene_size: SceneSize,
}

impl SizeConstraints {
    pub fn shrink_by(&self, size: ElementSize) -> Self {
        Self {
            min_size: self.min_size.shrink_by(size),
            max_size: self.max_size.shrink_by(size),
            scene_size: self.scene_size,
        }
    }

    pub fn grow_by(&self, size: ElementSize) -> Self {
        Self {
            min_size: self.min_size.grow_by(size),
            max_size: self.max_size.grow_by(size),
            scene_size: self.scene_size,
        }
    }
}
