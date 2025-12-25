use crate::{Element, Size, SizeConstraints};
use vn_vttrpg_window::{BoxPrimitive, Color, Scene};

/// Specifies where a child element should be anchored within its container.
pub enum AnchorLocation {
    TOP,
    BOTTOM,
    LEFT,
    RIGHT,

    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,

    CENTER,
}

/// A container that anchors its child to a specific location.
pub struct Anchor {
    child: Box<dyn Element>,
    child_size: Size,
    location: AnchorLocation,
}

impl Anchor {
    pub fn new(child: Box<dyn Element>, location: AnchorLocation) -> Self {
        Self {
            child,
            child_size: Size::ZERO,
            location,
        }
    }
}

impl Element for Anchor {
    fn layout(&mut self, constraints: SizeConstraints) -> Size {
        self.child_size = self.child.layout(constraints);
        constraints.max_size
    }

    fn draw(&mut self, origin: (f32, f32), size: Size, scene: &mut Scene) {
        match self.location {
            AnchorLocation::TOP => {
                self.child.draw(
                    (
                        origin.0 + size.width / 2.0 - self.child_size.width / 2.0,
                        origin.1,
                    ),
                    self.child_size,
                    scene,
                );
            }
            AnchorLocation::BOTTOM => {
                self.child.draw(
                    (
                        origin.0 + size.width / 2.0 - self.child_size.width / 2.0,
                        origin.1 + (size.height - self.child_size.height),
                    ),
                    self.child_size,
                    scene,
                );
            }
            AnchorLocation::LEFT => {
                self.child.draw(
                    (
                        origin.0,
                        origin.1 + size.height / 2.0 - self.child_size.height / 2.0,
                    ),
                    self.child_size,
                    scene,
                );
            }
            AnchorLocation::RIGHT => {
                self.child.draw(
                    (
                        origin.0 + (size.width - self.child_size.width),
                        origin.1 + size.height / 2.0 - self.child_size.height / 2.0,
                    ),
                    self.child_size,
                    scene,
                );
            }

            AnchorLocation::TopLeft => {
                self.child.draw(origin, self.child_size, scene);
            }
            AnchorLocation::TopRight => {
                self.child.draw(
                    (origin.0 + size.width - self.child_size.width, origin.1),
                    self.child_size,
                    scene,
                );
            }
            AnchorLocation::BottomLeft => self.child.draw(
                (origin.0, origin.1 + size.height - self.child_size.height),
                self.child_size,
                scene,
            ),
            AnchorLocation::BottomRight => self.child.draw(
                (
                    origin.0 + size.width - self.child_size.width,
                    origin.1 + size.height - self.child_size.height,
                ),
                self.child_size,
                scene,
            ),

            AnchorLocation::CENTER => self.child.draw(
                (
                    origin.0 + size.width / 2.0 - self.child_size.width / 2.0,
                    origin.1 + size.height / 2.0 - self.child_size.height / 2.0,
                ),
                self.child_size,
                scene,
            ),
        }
    }
}

pub struct Border {
    child: Box<dyn Element>,
    child_size: Size,
    border_size: f32,
    border_color: Color,
    corner_radius: f32,
}

impl Border {
    pub fn new(
        child: Box<dyn Element>,
        border_size: f32,
        corner_radius: f32,
        border_color: Color,
    ) -> Self {
        Self {
            child,
            child_size: Size::ZERO,
            border_size,
            border_color,
            corner_radius,
        }
    }
}

impl Element for Border {
    fn layout(&mut self, constraints: SizeConstraints) -> Size {
        let mut child_constraints = constraints;
        child_constraints.max_size.width -= self.border_size * 2.0;
        child_constraints.max_size.height -= self.border_size * 2.0;
        child_constraints.min_size.width =
            child_constraints.min_size.width.max(self.border_size * 2.0) - self.border_size * 2.0;
        child_constraints.min_size.height = child_constraints
            .min_size
            .height
            .max(self.border_size * 2.0)
            - self.border_size * 2.0;

        self.child_size = self.child.layout(child_constraints);
        Size {
            width: self.child_size.width + self.border_size * 2.0,
            height: self.child_size.height + self.border_size * 2.0,
        }
        .clamp_to_constraints(constraints)
    }

    fn draw(&mut self, origin: (f32, f32), size: Size, scene: &mut Scene) {
        scene.add_box(
            BoxPrimitive::builder()
                .transform(|t| t.translation([origin.0, origin.1]))
                .border_color(self.border_color)
                .corner_radius(self.corner_radius)
                .border_thickness(self.border_size)
                .size([size.width, size.height])
                .build(),
        );

        self.child.draw(
            (origin.0 + self.border_size, origin.1 + self.border_size),
            Size {
                width: size.width.max(self.border_size * 2.0) - self.border_size * 2.0,
                height: size.height.max(self.border_size * 2.0) - self.border_size * 2.0,
            },
            scene,
        );
    }
}
