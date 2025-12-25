#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Size = Size {
        width: 0.0,
        height: 0.0,
    };

    pub fn clamp_to_constraints(self, constraints: SizeConstraints) -> Size {
        Size {
            width: self
                .width
                .min(constraints.max_size.width)
                .max(constraints.min_size.width),
            height: self
                .height
                .min(constraints.max_size.height)
                .max(constraints.min_size.height),
        }
    }
}

/// Defines the minimum and maximum size constraints for layout.
#[derive(Debug, Clone, Copy)]
pub struct SizeConstraints {
    pub min_size: Size,
    pub max_size: Size,
}
