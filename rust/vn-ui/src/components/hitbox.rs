use crate::utils::ToArray;
use crate::{Element, ElementImpl, ElementSize, SizeConstraints, UiContext};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::{Rect, Scene};
use vn_ui_definitions::{Component, ui_component};

pub struct ExtendedHitboxParams<State, Message> {
    pub child: Rc<RefCell<dyn Element<State = State, Message = Message>>>,
}

ui_component!(ExtendedHitbox<ExtendedHitboxParams<State, Msg>>);

impl<State, Message: Clone> Component for ExtendedHitbox<State, Message> {
    type State = State;
    type Message = Message;
    type Params = ExtendedHitboxParams<State, Message>;

    fn layout(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        params: &Self::Params,
        constraints: SizeConstraints,
    ) -> ElementSize {
        params
            .child
            .borrow_mut()
            .layout(ctx, state, constraints)
            .clamp_to_constraints(constraints)
    }

    fn draw(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        params: &Self::Params,
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
            }
            .intersect(&ctx.clip_rect),
            |ctx| {
                params
                    .child
                    .borrow_mut()
                    .draw(ctx, state, origin, size, canvas);
            },
        );
    }

    fn handle_event(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        params: &Self::Params,
        event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        params.child.borrow_mut().handle_event(ctx, state, event)
    }
}
