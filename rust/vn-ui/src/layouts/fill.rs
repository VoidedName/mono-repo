use crate::{Element, ElementId, ElementImpl, ElementSize, SizeConstraints, UiContext};

use vn_scene::Scene;

pub struct Fill<State> {
    id: ElementId,
    element: Box<dyn Element<State = State>>,
}

impl<State> Fill<State> {
    pub fn new(element: Box<dyn Element<State = State>>, ctx: &mut UiContext) -> Self {
        Self {
            id: ctx.event_manager.borrow_mut().next_id(),
            element,
        }
    }
}

impl<State> ElementImpl for Fill<State> {
    type State = State;

    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        constraints: SizeConstraints,
    ) -> ElementSize {
        let child_size = self.element.layout(ctx, state, constraints);

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
            let new_size = self.element.layout(ctx, state, new_constraints);
            desired_size = new_size.clamp_to_constraints(constraints);
        }

        desired_size
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
    ) {
        self.element.draw(ctx, state, origin, size, canvas);
    }
}
