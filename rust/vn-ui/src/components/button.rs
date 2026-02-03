use crate::Scene;
use crate::utils::ToArray;
use crate::{
    Element, ElementId, ElementImpl, ElementSize, ElementWorld, EventHandler, InteractionEventKind,
    InteractionState, SizeConstraints, StateToParams, UiContext, into_box_impl,
};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::{BoxPrimitiveData, Color, Rect, Transform};
use vn_ui_definitions::Component;
use vn_ui_macros::UiElement;

#[derive(Debug, Copy, Clone)]
pub enum ButtonAction {
    Clicked,
}

pub struct ButtonParams<Message> {
    pub background: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub corner_radius: f32,
    pub interaction: InteractionState,
    pub on_click: EventHandler<ButtonAction, Message>,
}

#[derive(UiElement)]
pub struct Button<State: 'static, Message: Clone + 'static> {
    #[id]
    id: ElementId,
    child: Box<dyn Element<State = State, Message = Message>>,
    #[params]
    params: StateToParams<State, ButtonParams<Message>>,
}

impl<State, Message: Clone> Button<State, Message> {
    pub fn new<P: Into<StateToParams<State, ButtonParams<Message>>>>(
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

impl<State, Message: Clone> Component for Button<State, Message> {
    type State = State;
    type Message = Message;
    type Params = ButtonParams<Message>;

    fn layout(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        params: &Self::Params,
        constraints: SizeConstraints,
    ) -> ElementSize {
        let child_constraints = constraints.shrink_by(ElementSize {
            width: params.border_width * 2.0,
            height: params.border_width * 2.0,
        });

        self.child
            .layout(ctx, state, child_constraints)
            .grow_by(ElementSize {
                width: params.border_width * 2.0,
                height: params.border_width * 2.0,
            })
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
        let background = params.background;
        let border_color = params.border_color;

        let clip = Rect {
            position: origin.to_array(),
            size: size.to_array(),
        }
        .intersect(&ctx.clip_rect);

        ctx.with_hitbox_hierarchy(self.id, canvas.current_layer_id(), clip, |ctx| {
            canvas.add_box(BoxPrimitiveData {
                transform: Transform {
                    translation: [origin.0, origin.1],
                    ..Transform::DEFAULT
                },
                size: [size.width, size.height],
                color: background,
                border_color,
                border_thickness: params.border_width,
                border_radius: params.corner_radius,
                clip_rect: clip,
            });

            let margin = params.border_width * 2.0;
            self.child.draw(
                ctx,
                state,
                (
                    origin.0 + params.border_width,
                    origin.1 + params.border_width,
                ),
                size.shrink_by(ElementSize {
                    width: margin,
                    height: margin,
                }),
                canvas,
            );
        });
    }

    fn handle_event(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        params: &Self::Params,
        event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        let mut messages = self.child.handle_event(ctx, state, event);

        if event.target == Some(self.id) {
            messages.extend(params.on_click.handle(self.id, event, || match event.kind {
                InteractionEventKind::Click { .. } => {
                    vec![ButtonAction::Clicked]
                }
                _ => vec![],
            }));
        }

        messages
    }
}

pub trait ButtonExt<State, Message: Clone> {
    fn button<P: Into<StateToParams<State, ButtonParams<Message>>>>(
        self,
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Button<State, Message>;
}

impl<State, Message: Clone, E: Into<Box<dyn Element<State = State, Message = Message>>> + 'static>
    ButtonExt<State, Message> for E
{
    fn button<P: Into<StateToParams<State, ButtonParams<Message>>>>(
        self,
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Button<State, Message> {
        Button::new(self, params, world)
    }
}

into_box_impl!(Button);
