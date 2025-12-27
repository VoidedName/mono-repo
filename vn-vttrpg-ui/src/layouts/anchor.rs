use crate::{ConcreteSize, Element, SizeConstraints, UiContext};
use vn_vttrpg_window::Scene;

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub struct AnchorParams {
    pub location: AnchorLocation,
}

pub struct Anchor {
    child: Box<dyn Element>,
    child_size: ConcreteSize,
    params: AnchorParams,
}

impl Anchor {
    pub fn new(child: Box<dyn Element>, params: AnchorParams) -> Self {
        Self {
            child,
            child_size: ConcreteSize::ZERO,
            params,
        }
    }
}

impl Element for Anchor {
    fn layout(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize {
        let mut child_constraints = constraints;
        child_constraints.min_size = ConcreteSize::ZERO;

        self.child_size = self.child.layout(ctx, child_constraints);

        ConcreteSize {
            width: constraints.max_size.width.unwrap_or(self.child_size.width),
            height: constraints
                .max_size
                .height
                .unwrap_or(self.child_size.height),
        }
        .clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ConcreteSize,
        scene: &mut Scene,
    ) {
        match self.params.location {
            AnchorLocation::TOP => self.child.draw(
                ctx,
                (origin.0 + size.width / 2.0 - self.child_size.width / 2.0, origin.1),
                self.child_size,
                scene,
            ),
            AnchorLocation::BOTTOM => self.child.draw(
                ctx,
                (
                    origin.0 + size.width / 2.0 - self.child_size.width / 2.0,
                    origin.1 + size.height - self.child_size.height,
                ),
                self.child_size,
                scene,
            ),
            AnchorLocation::LEFT => self.child.draw(
                ctx,
                (origin.0, origin.1 + size.height / 2.0 - self.child_size.height / 2.0),
                self.child_size,
                scene,
            ),
            AnchorLocation::RIGHT => self.child.draw(
                ctx,
                (
                    origin.0 + size.width - self.child_size.width,
                    origin.1 + size.height / 2.0 - self.child_size.height / 2.0,
                ),
                self.child_size,
                scene,
            ),
            AnchorLocation::TopLeft => self.child.draw(ctx, origin, self.child_size, scene),
            AnchorLocation::TopRight => self.child.draw(
                ctx,
                (origin.0 + size.width - self.child_size.width, origin.1),
                self.child_size,
                scene,
            ),
            AnchorLocation::BottomLeft => self.child.draw(
                ctx,
                (origin.0, origin.1 + size.height - self.child_size.height),
                self.child_size,
                scene,
            ),
            AnchorLocation::BottomRight => self.child.draw(
                ctx,
                (
                    origin.0 + size.width - self.child_size.width,
                    origin.1 + size.height - self.child_size.height,
                ),
                self.child_size,
                scene,
            ),
            AnchorLocation::CENTER => self.child.draw(
                ctx,
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
