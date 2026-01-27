use crate::{
    Element, ElementId, ElementImpl, ElementSize, ElementWorld, InteractionEvent, SizeConstraints,
    StateToParams, UiContext, into_box_impl,
};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::Scene;

pub struct Interactive<State: 'static, Message: 'static> {
    id: ElementId,
    child: Box<dyn Element<State = State, Message = Message>>,
    params: StateToParams<State, InteractiveParams>,
}

pub struct InteractiveParams {
    pub is_interactive: bool,
}

impl<State, Message> Interactive<State, Message> {
    pub fn new<P: Into<StateToParams<State, InteractiveParams>>>(
        child: impl Into<Box<dyn Element<State = State, Message = Message>>>,
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Self {
        Self {
            id: world.borrow_mut().next_id(),
            child: child.into(),
            params: params.into(),
        }
    }
}

impl<State, Message> ElementImpl for Interactive<State, Message> {
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
        let _params = self.params.call(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });
        self.child.layout(ctx, state, constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
    ) {
        let params = self.params.call(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });
        ctx.with_interactivity(params.is_interactive, |ctx| {
            self.child.draw(ctx, state, origin, size, canvas);
        });
    }

    fn handle_event_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        event: &InteractionEvent,
    ) -> Vec<Self::Message> {
        self.child.handle_event(ctx, state, event)
    }
}

pub trait InteractiveExt<State, Message> {
    fn interactive<P: Into<StateToParams<State, InteractiveParams>>>(
        self,
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Interactive<State, Message>;

    fn interactive_set(
        self,
        interactive: bool,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Interactive<State, Message>;
}

impl<State, Message, E: Into<Box<dyn Element<State = State, Message = Message>>> + 'static>
    InteractiveExt<State, Message> for E
{
    fn interactive<P: Into<StateToParams<State, InteractiveParams>>>(
        self,
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Interactive<State, Message> {
        Interactive::new(self, params, world)
    }

    fn interactive_set(
        self,
        interactive: bool,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Interactive<State, Message> {
        let params = StateToParams(Box::new(move |_| InteractiveParams {
            is_interactive: interactive,
        }));

        Interactive::new(self, params, world)
    }
}

into_box_impl!(Interactive);
