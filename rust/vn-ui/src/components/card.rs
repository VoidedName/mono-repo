use crate::{
    Element, ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints, StateToParams,
    UiContext,
};
use vn_scene::{BoxPrimitiveData, Color, Scene, Transform};
use vn_ui_animation_macros::Interpolatable;
use vn_utils::option::UpdateOption;

#[derive(Clone, Copy, Interpolatable)]
pub struct CardParams {
    pub background_color: Color,
    pub border_size: f32,
    pub border_color: Color,
    pub corner_radius: f32,
}

pub struct Card<State, Message> {
    id: ElementId,
    child: Box<dyn Element<State = State, Message = Message>>,
    params: StateToParams<State, CardParams>,
}

impl<State, Message> Card<State, Message> {
    pub fn new(
        child: Box<dyn Element<State = State, Message = Message>>,
        params: StateToParams<State, CardParams>,
        world: &mut ElementWorld,
    ) -> Self {
        Self {
            id: world.next_id(),
            child,
            params,
        }
    }
}

// Actually, the Card should probably manage its Padding internal params based on its own params.
// But Padding now also uses StateToParams.

impl<State, Message> ElementImpl for Card<State, Message> {
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
        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        let mut child_constraints = constraints;
        let padding = params.border_size;
        let x_padding = padding * 2.0;
        let y_padding = padding * 2.0;

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
        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        canvas.add_box(BoxPrimitiveData {
            transform: Transform {
                translation: [origin.0, origin.1],
                ..Transform::DEFAULT
            },
            size: [size.width, size.height],
            color: params.background_color,
            border_color: params.border_color,
            border_thickness: params.border_size,
            border_radius: params.corner_radius,
            clip_rect: ctx.clip_rect,
        });

        let padding = params.border_size;
        self.child.draw(
            ctx,
            state,
            (origin.0 + padding, origin.1 + padding),
            ElementSize {
                width: size.width.max(padding * 2.0) - padding * 2.0,
                height: size.height.max(padding * 2.0) - padding * 2.0,
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

pub trait CardExt: Element {
    fn card(
        self,
        params: StateToParams<Self::State, CardParams>,
        world: &mut ElementWorld,
    ) -> Card<Self::State, Self::Message>;
}

impl<E: Element + 'static> CardExt for E {
    fn card(
        self,
        params: StateToParams<Self::State, CardParams>,
        world: &mut ElementWorld,
    ) -> Card<Self::State, Self::Message> {
        Card::new(Box::new(self), params, world)
    }
}
