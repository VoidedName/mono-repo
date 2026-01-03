use crate::{Element, ElementId, ElementImpl, ElementSize, SizeConstraints, UiContext};

pub struct Fill {
    id: ElementId,
    element: Box<dyn Element>,
}

impl Fill {
    pub fn new(element: Box<dyn Element>, ctx: &mut UiContext) -> Self {
        Self {
            id: ctx.event_manager.next_id(),
            element,
        }
    }
}

impl ElementImpl for Fill {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        let child_size = self.element.layout(ctx, constraints);

        let height = match constraints.max_size.height {
            Some(h) => h,
            _ => child_size.height,
        };

        let width = match constraints.max_size.width {
            Some(w) => w,
            _ => child_size.width,
        };

        let mut desired_size = ElementSize { width, height }.clamp_to_constraints(constraints);

        if width > desired_size.width {
            let mut new_constraints = constraints;
            new_constraints.max_size.width = Some(desired_size.width);
            let new_size = self.element.layout(ctx, new_constraints);
            desired_size = new_size.clamp_to_constraints(constraints);
        }

        desired_size
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut vn_window::Scene,
    ) {
        self.element.draw(ctx, origin, size, scene);
    }
}
