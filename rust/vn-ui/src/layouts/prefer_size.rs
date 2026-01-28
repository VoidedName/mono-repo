use crate::{
    DynamicDimension, DynamicSize, Element, ElementId, ElementImpl, ElementSize, ElementWorld,
    InteractionEvent, SizeConstraints, StateToParams, StateToParamsArgs, UiContext, into_box_impl,
};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::Scene;

pub struct PreferSizeParams {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

pub struct PreferSize<State: 'static, Message> {
    id: ElementId,
    child: Box<dyn Element<State = State, Message = Message>>,
    params: StateToParams<State, PreferSizeParams>,
}

impl<State: 'static, Message> PreferSize<State, Message> {
    pub fn new<P: Into<StateToParams<State, PreferSizeParams>>>(
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

impl<State, Message> ElementImpl for PreferSize<State, Message> {
    type State = State;
    type Message = Message;

    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        mut constraints: SizeConstraints,
    ) -> ElementSize {
        let params = self.params.call(StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        constraints.min_size = ElementSize {
            width: params.width.unwrap_or(0.0),
            height: params.height.unwrap_or(0.0),
        }
        .clamp_to_constraints(constraints);

        constraints.max_size = DynamicSize {
            width: match params.width {
                Some(width) => match constraints.max_size.width {
                    DynamicDimension::Hint(_) => DynamicDimension::Limit(width),
                    DynamicDimension::Limit(limit) => DynamicDimension::Limit(width.min(limit)),
                },
                None => constraints.max_size.width,
            },
            height: match params.height {
                Some(height) => match constraints.max_size.height {
                    DynamicDimension::Hint(_) => DynamicDimension::Limit(height),
                    DynamicDimension::Limit(limit) => DynamicDimension::Limit(height.min(limit)),
                },
                None => constraints.max_size.height,
            },
        };

        let size = self.child.layout(ctx, &state, constraints);

        size
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut dyn Scene,
    ) {
        self.child.draw(ctx, state, origin, size, scene);
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

pub trait PreferSizeExt<State, Message> {
    fn prefer_size<P: Into<StateToParams<State, PreferSizeParams>>>(
        self,
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> PreferSize<State, Message>;
}

impl<State, Message, E: Into<Box<dyn Element<State = State, Message = Message>>> + 'static>
    PreferSizeExt<State, Message> for E
{
    fn prefer_size<P: Into<StateToParams<State, PreferSizeParams>>>(
        self,
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> PreferSize<State, Message> {
        PreferSize::new(self, params, world)
    }
}

into_box_impl!(PreferSize);
