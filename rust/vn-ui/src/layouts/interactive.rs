use crate::{
    Element, ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints, StateToParams,
    UiContext,
};
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
        world: &mut ElementWorld,
    ) -> Self {
        Self {
            id: world.next_id(),
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

    fn layout_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        constraints: SizeConstraints,
    ) -> ElementSize {
        let _params = (self.params)(crate::StateToParamsArgs {
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
        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });
        ctx.with_interactivity(params.is_interactive, |ctx| {
            self.child.draw(ctx, state, origin, size, canvas);
        });
    }
}

pub trait InteractiveExt: Element {
    fn interactive(
        self,
        params: StateToParams<Self::State, InteractiveParams>,
        world: &mut ElementWorld,
    ) -> Interactive<Self::State>;

    fn interactive_set(
        self,
        interactive: bool,
        world: &mut ElementWorld,
    ) -> Interactive<Self::State>;
}

impl<E: Element + 'static> InteractiveExt for E {
    fn interactive(
        self,
        params: StateToParams<Self::State, InteractiveParams>,
        world: &mut ElementWorld,
    ) -> Interactive<Self::State> {
        Interactive::new(Box::new(self), params, world)
    }

    fn interactive_set(
        self,
        interactive: bool,
        world: &mut ElementWorld,
    ) -> Interactive<Self::State> {
        Interactive::new(
            Box::new(self),
            Box::new(move |_| InteractiveParams {
                is_interactive: interactive,
            }),
            world,
        )
    }
}
