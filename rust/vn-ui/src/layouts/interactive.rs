use crate::{Element, ElementId, ElementImpl, ElementSize, SizeConstraints, StateToParams, UiContext};
use vn_scene::Scene;

pub struct Interactive<State> {
    id: ElementId,
    child: Box<dyn Element<State = State>>,
    params: StateToParams<State, InteractiveParams>,
}

pub struct InteractiveParams {
    pub is_interactive: bool,
}

impl<State> Interactive<State> {
    pub fn new(
        child: Box<dyn Element<State = State>>,
        params: StateToParams<State, InteractiveParams>,
        ctx: &mut UiContext,
    ) -> Self {
        Self {
            id: ctx.event_manager.next_id(),
            child,
            params,
        }
    }
}

impl<State> ElementImpl for Interactive<State> {
    type State = State;

    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, state: &Self::State, constraints: SizeConstraints) -> ElementSize {
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
        let params = (self.params)(state, &ctx.now);
        ctx.with_interactivity(params.is_interactive, |ctx| {
            self.child.draw(ctx, state, origin, size, canvas);
        });
    }
}

