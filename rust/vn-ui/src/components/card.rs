use crate::utils::ToArray;
use crate::{
    Element, ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints, StateToParams,
    UiContext,
};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::{BoxPrimitiveData, Color, Rect, Scene, Transform};
use vn_ui_animation_macros::Interpolatable;
use vn_ui_definitions::Component;
use vn_ui_macros::UiElement;

#[derive(Clone, Copy, Interpolatable)]
pub struct CardParams {
    pub background_color: Color,
    pub border_size: f32,
    pub border_color: Color,
    pub corner_radius: f32,
}

#[derive(UiElement)]
pub struct Card<State: 'static, Message: Clone + 'static> {
    #[id]
    id: ElementId,
    child: Box<dyn Element<State = State, Message = Message>>,
    #[params]
    params: StateToParams<State, CardParams>,
}

impl<State, Message: Clone> Card<State, Message> {
    pub fn new<P: Into<StateToParams<State, CardParams>>>(
        child: impl Into<Box<dyn Element<State = State, Message = Message>>>,
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Self {
        Self {
            id: world.borrow_mut().next_id(),
            child: child.into(),
            params: params.into(),
        }
    }
}

// Actually, the Card should probably manage its Padding internal params based on its own params.
// But Padding now also uses StateToParams.

impl<State, Message: Clone> Component for Card<State, Message> {
    type State = State;
    type Message = Message;
    type Params = CardParams;

    fn layout(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        params: &Self::Params,
        constraints: SizeConstraints,
    ) -> ElementSize {
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

    fn draw(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        params: &Self::Params,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
    ) {
        let clip = Rect {
            position: origin.to_array(),
            size: size.to_array(),
        }
        .intersect(&ctx.clip_rect);

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
            clip_rect: clip,
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

    fn handle_event(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        _params: &Self::Params,
        event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        self.child.handle_event(ctx, state, event)
    }
}

pub trait CardExt<State, Message: Clone> {
    fn card<P: Into<StateToParams<State, CardParams>>>(
        self,
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Card<State, Message>;
}

impl<State, Message: Clone, E: Into<Box<dyn Element<State = State, Message = Message>>> + 'static>
    CardExt<State, Message> for E
{
    fn card<P: Into<StateToParams<State, CardParams>>>(
        self,
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Card<State, Message> {
        Card::new(self, params, world)
    }
}
