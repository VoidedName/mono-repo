use crate::{
    Element, ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints, StateToParams,
    UiContext,
};
use vn_scene::Scene;

#[derive(Clone, Copy)]
pub enum AnchorLocation {
    TOP,
    BOTTOM,
    LEFT,
    RIGHT,

    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,

    CENTER,
}

#[derive(Clone, Copy)]
pub struct AnchorParams {
    pub location: AnchorLocation,
}

pub struct Anchor<State> {
    id: ElementId,
    child: Box<dyn Element<State = State>>,
    child_size: ElementSize,
    params: StateToParams<State, AnchorParams>,
}

impl<State> Anchor<State> {
    pub fn new(
        child: Box<dyn Element<State = State>>,
        params: StateToParams<State, AnchorParams>,
        world: &mut ElementWorld,
    ) -> Self {
        Self {
            id: world.next_id(),
            child,
            child_size: ElementSize::ZERO,
            params,
        }
    }
}

impl<State> ElementImpl for Anchor<State> {
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

        let mut child_constraints = constraints;
        child_constraints.min_size = ElementSize::ZERO;

        self.child_size = self.child.layout(ctx, state, child_constraints);

        ElementSize {
            width: constraints.max_size.width.unwrap_or(self.child_size.width),
            height: constraints
                .max_size
                .height
                .unwrap_or(self.child_size.height),
        }
        .clamp_to_constraints(constraints)
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
        match params.location {
            AnchorLocation::TOP => self.child.draw(
                ctx,
                state,
                (
                    origin.0 + size.width / 2.0 - self.child_size.width / 2.0,
                    origin.1,
                ),
                self.child_size,
                canvas,
            ),
            AnchorLocation::BOTTOM => self.child.draw(
                ctx,
                state,
                (
                    origin.0 + size.width / 2.0 - self.child_size.width / 2.0,
                    origin.1 + size.height - self.child_size.height,
                ),
                self.child_size,
                canvas,
            ),
            AnchorLocation::LEFT => self.child.draw(
                ctx,
                state,
                (
                    origin.0,
                    origin.1 + size.height / 2.0 - self.child_size.height / 2.0,
                ),
                self.child_size,
                canvas,
            ),
            AnchorLocation::RIGHT => self.child.draw(
                ctx,
                state,
                (
                    origin.0 + size.width - self.child_size.width,
                    origin.1 + size.height / 2.0 - self.child_size.height / 2.0,
                ),
                self.child_size,
                canvas,
            ),
            AnchorLocation::TopLeft => self.child.draw(ctx, state, origin, self.child_size, canvas),
            AnchorLocation::TopRight => self.child.draw(
                ctx,
                state,
                (origin.0 + size.width - self.child_size.width, origin.1),
                self.child_size,
                canvas,
            ),
            AnchorLocation::BottomLeft => self.child.draw(
                ctx,
                state,
                (origin.0, origin.1 + size.height - self.child_size.height),
                self.child_size,
                canvas,
            ),
            AnchorLocation::BottomRight => self.child.draw(
                ctx,
                state,
                (
                    origin.0 + size.width - self.child_size.width,
                    origin.1 + size.height - self.child_size.height,
                ),
                self.child_size,
                canvas,
            ),
            AnchorLocation::CENTER => self.child.draw(
                ctx,
                state,
                (
                    origin.0 + size.width / 2.0 - self.child_size.width / 2.0,
                    origin.1 + size.height / 2.0 - self.child_size.height / 2.0,
                ),
                self.child_size,
                canvas,
            ),
        }
    }
}

pub trait AnchorExt: Element {
    fn anchor(
        self,
        params: StateToParams<Self::State, AnchorParams>,
        world: &mut ElementWorld,
    ) -> Anchor<Self::State>;
}

impl<E: Element + 'static> AnchorExt for E {
    fn anchor(
        self,
        params: StateToParams<Self::State, AnchorParams>,
        world: &mut ElementWorld,
    ) -> Anchor<Self::State> {
        Anchor::new(Box::new(self), params, world)
    }
}
