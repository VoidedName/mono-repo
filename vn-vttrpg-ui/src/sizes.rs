use vn_vttrpg_window::scene::SceneSize;

/// A concrete size with a fixed width and height.
#[derive(Debug, Clone, Copy)]
pub struct ConcreteSize {
    pub width: f32,
    pub height: f32,
}

impl ConcreteSize {
    pub const ZERO: ConcreteSize = ConcreteSize {
        width: 0.0,
        height: 0.0,
    };

    pub fn clamp_to_constraints(self, constraints: SizeConstraints) -> ConcreteSize {
        let max_size = constraints.max_size.to_concrete();

        ConcreteSize {
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
#[derive(Debug, Clone, Copy)]
pub struct DynamicSize {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl DynamicSize {
    pub fn to_concrete(self) -> ConcreteSize {
        ConcreteSize {
            width: self.width.unwrap_or(f32::INFINITY),
            height: self.height.unwrap_or(f32::INFINITY),
        }
    }
}

/// Defines the minimum and maximum size constraints for layout.
#[derive(Debug, Clone, Copy)]
pub struct SizeConstraints {
    pub min_size: ConcreteSize,
    pub max_size: DynamicSize,
    pub scene_size: SceneSize,
}
