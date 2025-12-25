use crate::{ConcreteSize, DynamicSize, Element, SizeConstraints};
use vn_utils::UpdateOption;
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
    child_size: ConcreteSize,
    location: AnchorLocation,
}

impl Anchor {
    pub fn new(child: Box<dyn Element>, location: AnchorLocation) -> Self {
        Self {
            child,
            child_size: ConcreteSize::ZERO,
            location,
        }
    }
}

impl Element for Anchor {
    fn layout(&mut self, constraints: SizeConstraints) -> ConcreteSize {
        self.child_size = self.child.layout(constraints);

        // if a component has no constraints, use its child's size
        let width = match constraints.max_size.width {
            Some(w) => w,
            None => self.child_size.width,
        };

        let height = match constraints.max_size.height {
            Some(w) => w,
            None => self.child_size.height,
        };

        ConcreteSize { width, height }
    }

    fn draw_impl(&mut self, origin: (f32, f32), size: ConcreteSize, scene: &mut Scene) {
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
    child_size: ConcreteSize,
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
            child_size: ConcreteSize::ZERO,
            border_size,
            border_color,
            corner_radius,
        }
    }
}

impl Element for Border {
    fn layout(&mut self, constraints: SizeConstraints) -> ConcreteSize {
        let mut child_constraints = constraints;
        let margin = self.border_size * 2.0;

        child_constraints
            .max_size
            .width
            .update(|w| w.max(margin) - margin);
        child_constraints
            .max_size
            .height
            .update(|h| h.max(margin) - margin);

        child_constraints.min_size.width = child_constraints.min_size.width.max(margin) - margin;
        child_constraints.min_size.height = child_constraints.min_size.height.max(margin) - margin;

        self.child_size = self.child.layout(child_constraints);
        ConcreteSize {
            width: self.child_size.width + margin,
            height: self.child_size.height + margin,
        }
        .clamp_to_constraints(constraints)
    }

    fn draw_impl(&mut self, origin: (f32, f32), size: ConcreteSize, scene: &mut Scene) {
        let margin = self.border_size * 2.0;

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
            ConcreteSize {
                width: size.width.max(margin) - margin,
                height: size.height.max(margin) - margin,
            },
            scene,
        );
    }
}

pub struct Flex {
    children: Vec<Box<dyn Element>>,
    layout: Vec<ConcreteSize>,
    // direction?
}

impl Flex {
    pub fn new(children: Vec<Box<dyn Element>>) -> Self {
        Self {
            layout: std::iter::repeat(ConcreteSize::ZERO).take(children.len()).collect(),
            children,
        }
    }
}

impl Element for Flex {
    fn layout(&mut self, constraints: SizeConstraints) -> ConcreteSize {
        // what do we do with containers that grow? like anchor?
        // do we extend constraints to denote that they should not grow along some axis?

        let mut total_width: f32 = 0.0;
        let mut max_height: f32 = 0.0;

        for (idx, child) in self.children.iter_mut().enumerate() {
            let child_size = child.layout(SizeConstraints {
                min_size: ConcreteSize {
                    width: 0.0,
                    height: constraints.min_size.height,
                },
                max_size: DynamicSize {
                    width: None,
                    height: constraints.max_size.height,
                },
            });

            total_width += child_size.width;
            max_height = max_height.max(child_size.height);

            self.layout[idx] = child_size;
        }

        ConcreteSize {
            width: total_width,
            height: max_height
        }.clamp_to_constraints(constraints)
    }

    fn draw_impl(&mut self, origin: (f32, f32), _size: ConcreteSize, scene: &mut Scene) {
        // is this fine? we are ignoring the input size, assuming that we get something compatible
        // with what we reported in layout?
        let mut offset = origin.0;
        for (idx, child) in self.children.iter_mut().enumerate() {
            child.draw(
                (offset, origin.1),
                self.layout[idx],
                scene,
            );

            offset += self.layout[idx].width;
        }
    }
}
