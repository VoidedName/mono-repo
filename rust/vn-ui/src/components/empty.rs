use crate::{into_box_impl, ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints, UiContext};
use vn_scene::Scene;

pub struct Empty<State: 'static, Message: 'static> {
    id: ElementId,
    _ph: std::marker::PhantomData<(State, Message)>,
}

impl<State, Message> Empty<State, Message> {
    pub fn new(world: &mut ElementWorld) -> Self {
        Self {
            id: world.next_id(),
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
        _ctx: &mut UiContext,
        _state: &Self::State,
        constraints: SizeConstraints,
    ) -> ElementSize {
        constraints.min_size
    }

    fn draw_impl(
        &mut self,
        _ctx: &mut UiContext,
        _state: &Self::State,
        _origin: (f32, f32),
        _size: ElementSize,
        _scene: &mut dyn Scene,
    ) {
    }

    fn handle_event_impl(
        &mut self,
        _ctx: &mut UiContext,
        _state: &Self::State,
        _event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        vec![]
    }
}

into_box_impl!(Empty);