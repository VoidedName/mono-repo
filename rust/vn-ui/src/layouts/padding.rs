use crate::{
    Element, ElementId, ElementImpl, ElementSize, SizeConstraints, StateToParams, UiContext,
};
use vn_scene::Scene;
use vn_ui_animation_macros::Interpolatable;
use vn_utils::option::UpdateOption;

#[derive(Clone, Copy, Debug, Interpolatable)]
pub struct PaddingParams {
    pub pad_left: f32,
    pub pad_right: f32,
    pub pad_top: f32,
    pub pad_bottom: f32,
}

impl PaddingParams {
    pub fn uniform(value: f32) -> Self {
        Self {
            pad_left: value,
            pad_right: value,
            pad_top: value,
            pad_bottom: value,
        }
    }
}

pub struct Padding<State> {
    id: ElementId,
    child: Box<dyn Element<State = State>>,
    params: StateToParams<State, PaddingParams>,
    child_size: ElementSize,
}

impl<State> Padding<State> {
    pub fn new(
        child: Box<dyn Element<State = State>>,
        params: StateToParams<State, PaddingParams>,
        ctx: &mut UiContext,
    ) -> Self {
        Self {
            id: ctx.event_manager.next_id(),
            child,
            params,
            child_size: ElementSize::ZERO,
        }
    }
}

impl<State> ElementImpl for Padding<State> {
    type State = State;

    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, state: &Self::State, constraints: SizeConstraints) -> ElementSize {
        let params = (self.params)(state, &ctx.now);

        let mut child_constraints = constraints;
        let x_padding = params.pad_left + params.pad_right;
        let y_padding = params.pad_top + params.pad_bottom;

        child_constraints
            .max_size
            .width
            .update(|w| w.max(x_padding) - x_padding);
        child_constraints
            .max_size
            .height
            .update(|h| h.max(y_padding) - y_padding);

        child_constraints.min_size.width =
            child_constraints.min_size.width.max(x_padding) - x_padding;
        child_constraints.min_size.height =
            child_constraints.min_size.height.max(y_padding) - y_padding;

        self.child_size = self.child.layout(ctx, state, child_constraints);

        ElementSize {
            width: self.child_size.width + x_padding,
            height: self.child_size.height + y_padding,
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
        let params = (self.params)(state, &ctx.now);

        let x_padding = params.pad_left + params.pad_right;
        let y_padding = params.pad_top + params.pad_bottom;

        self.child.draw(
            ctx,
            state,
            (origin.0 + params.pad_left, origin.1 + params.pad_top),
            ElementSize {
                width: size.width.max(x_padding) - x_padding,
                height: size.height.max(y_padding) - y_padding,
            },
            canvas,
        );
    }
}

