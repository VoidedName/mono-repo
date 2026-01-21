use crate::utils::ToArray;
use crate::{
    Element, ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints, UiContext,
};
use vn_scene::{Rect, Scene};

pub struct ExtendedHitbox<State> {
    id: ElementId,
    element: Box<dyn Element<State = State>>,
}

impl<State> ExtendedHitbox<State> {
    pub fn new(element: Box<dyn Element<State = State>>, world: &mut ElementWorld) -> Self {
        let ui_id = world.next_id();
        Self { id: ui_id, element }
    }
}

impl<State> ElementImpl for ExtendedHitbox<State> {
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
        self.element
            .layout(ctx, state, constraints)
            .clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
    ) {
        ctx.with_hitbox_hierarchy(
            self.id,
            canvas.current_layer_id(),
            Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |ctx| {
                self.element.draw(ctx, state, origin, size, canvas);
            },
        );
    }
}
