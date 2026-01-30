use crate::{
    Element, ElementId, ElementImpl, ElementSize, ElementWorld, InteractionEvent, SizeConstraints,
    StateToParams, StateToParamsArgs, UiContext, into_box_impl,
};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::Scene;

pub struct ConditionalParams {
    pub show: bool,
}

pub struct Conditional<State: 'static, Message: 'static> {
    id: ElementId,
    child: Box<dyn Element<State = State, Message = Message>>,
    params: StateToParams<State, ConditionalParams>,
    _phantom: std::marker::PhantomData<Message>,
}

impl<State, Message> Conditional<State, Message> {
    pub fn new(
        child: Box<dyn Element<State = State, Message = Message>>,
        params: impl Into<StateToParams<State, ConditionalParams>>,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Self {
        Self {
            id: world.borrow_mut().next_id(),
            child,
            params: params.into(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<State, Message> ElementImpl for Conditional<State, Message> {
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
        let params = self.params.call(StateToParamsArgs {
            ctx,
            state,
            id: self.id,
        });

        if params.show {
            self.child.layout(ctx, state, constraints)
        } else {
            ElementSize::ZERO.clamp_to_constraints(constraints)
        }
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut dyn Scene,
    ) {
        let params = self.params.call(StateToParamsArgs {
            ctx,
            state,
            id: self.id,
        });

        if params.show {
            self.child.draw(ctx, state, origin, size, scene);
        }
    }

    fn handle_event_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        event: &InteractionEvent,
    ) -> Vec<Self::Message> {
        let params = self.params.call(StateToParamsArgs {
            ctx,
            state,
            id: self.id,
        });

        if params.show {
            self.child.handle_event(ctx, state, event)
        } else {
            vec![]
        }
    }
}

into_box_impl!(Conditional);
