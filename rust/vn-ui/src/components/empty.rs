use vn_scene::Scene;
use crate::{ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints, UiContext};

pub struct Empty<State> {
    id: ElementId,
    size: ElementSize,
    _ph: std::marker::PhantomData<State>,
}

impl<State> Empty<State> {
    pub fn new(size: ElementSize, world: &mut ElementWorld) -> Self {
        Self {
            id: world.next_id(),
            size,
            _ph: Default::default(),
        }
    }
}

impl<State> ElementImpl for Empty<State> {
    type State = State;

    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, state: &Self::State, constraints: SizeConstraints) -> ElementSize {
        self.size.clamp_to_constraints(constraints)
    }

    fn draw_impl(&mut self, ctx: &mut UiContext, state: &Self::State, origin: (f32, f32), size: ElementSize, canvas: &mut dyn Scene) {}
}