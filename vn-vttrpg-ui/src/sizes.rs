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
    pub fn clip_to_constraints(self, constraints: SizeConstraints) -> Size {
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

// should I support infinite size?
// scrollable areas technically have an infinite internal size along the scroll axis
// there are components that expand to fit the container size,
// and there are containers that shrink to their children
#[derive(Debug, Clone, Copy)]

pub struct SizeConstraints {
    pub min_size: Size,
    pub max_size: Size,
}
