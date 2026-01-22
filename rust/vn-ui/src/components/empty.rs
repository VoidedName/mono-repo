use crate::{ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints, UiContext};
use vn_scene::Scene;

pub struct Empty<State, Message> {
    id: ElementId,
    size: ElementSize,
    _ph: std::marker::PhantomData<(State, Message)>,
}

impl<State, Message> Empty<State, Message> {
    pub fn new(size: ElementSize, world: &mut ElementWorld) -> Self {
        Self {
            id: world.next_id(),
            size,
            _ph: Default::default(),
        }
    }
}

impl<State, Message> ElementImpl for Empty<State, Message> {
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
        self.size.clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
    ) {
    }

    fn handle_event_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        vec![]
    }
}
