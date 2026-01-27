use crate::{into_box_impl, Element, ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints, StateToParams, UiContext};
use vn_scene::Scene;
use vn_ui_animation_macros::Interpolatable;

#[derive(Clone, Copy, Debug, Interpolatable, Default)]
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

    pub fn horizontal(value: f32) -> Self {
        Self {
            pad_left: value / 2.0,
            pad_top: 0.0,
            pad_right: value / 2.0,
            pad_bottom: 0.0,
        }
    }

    pub fn vertical(value: f32) -> Self {
        Self {
            pad_top: value / 2.0,
            pad_left: 0.0,
            pad_bottom: value / 2.0,
            pad_right: 0.0,
        }
    }
}

pub struct Padding<State: 'static, Message: 'static> {
    id: ElementId,
    child: Box<dyn Element<State = State, Message = Message>>,
    params: StateToParams<State, PaddingParams>,
}

impl<State, Message> Padding<State, Message> {
    pub fn new<P: Into<StateToParams<State, PaddingParams>>>(
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

impl<State, Message> ElementImpl for Padding<State, Message> {
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
        let params = self.params.call(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

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

        let child_size = self.child.layout(ctx, state, child_constraints);

        ElementSize {
            width: child_size.width + x_padding,
            height: child_size.height + y_padding,
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
        let params = self.params.call(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

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

    fn handle_event_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        self.child.handle_event(ctx, state, event)
    }
}

pub trait PaddingExt<State, Message> {
    fn padding<P: Into<StateToParams<State, PaddingParams>>>(
        self,
        params: P,
        world: &mut ElementWorld,
    ) -> Padding<State, Message>;
}

impl<State, Message, E: Into<Box<dyn Element<State = State, Message = Message>>> + 'static> PaddingExt<State, Message> for E {
    fn padding<P: Into<StateToParams<State, PaddingParams>>>(
        self,
        params: P,
        world: &mut ElementWorld,
    ) -> Padding<State, Message> {
        Padding::new(self, params, world)
    }
}

into_box_impl!(Padding);