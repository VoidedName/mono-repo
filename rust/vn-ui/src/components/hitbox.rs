use crate::utils::ToArray;
use crate::{
    Element, ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints, UiContext,
    into_box_impl,
};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::{Rect, Scene};

pub struct ExtendedHitbox<State, Message> {
    id: ElementId,
    element: Box<dyn Element<State = State, Message = Message>>,
}

impl<State, Message> ExtendedHitbox<State, Message> {
    pub fn new(
        element: impl Into<Box<dyn Element<State = State, Message = Message>>>,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Self {
        let ui_id = world.borrow_mut().next_id();
        Self {
            id: ui_id,
            element: element.into(),
        }
    }
}

impl<State, Message> ElementImpl for ExtendedHitbox<State, Message> {
    type State = State;
    type Message = Message;

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

    fn handle_event_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        self.element.handle_event(ctx, state, event)
    }
}

into_box_impl!(ExtendedHitbox);
