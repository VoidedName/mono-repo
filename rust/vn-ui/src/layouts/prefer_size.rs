use crate::{into_box_impl, DynamicDimension, DynamicSize, Element, ElementId, ElementImpl, ElementSize, ElementWorld, InteractionEvent, SizeConstraints, StateToParams, StateToParamsArgs, UiContext};
use vn_scene::Scene;

pub struct PreferSizeParams {
    pub size: ElementSize,
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
        world: &mut ElementWorld,
    ) -> Self {
        Self {
            id: world.next_id(),
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

        let size = params.size.clamp_to_constraints(constraints);

        constraints.min_size = size;
        constraints.max_size = DynamicSize {
            width: DynamicDimension::Limit(size.width),
            height: DynamicDimension::Limit(size.height),
        };

        self.child.layout(ctx, &state, constraints);

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

into_box_impl!(PreferSize);