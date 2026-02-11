use crate::Scene;
use crate::utils::ToArray;
use crate::{
    ElementImpl, ElementSize, EventHandler, InteractionEventKind, InteractionState,
    SizeConstraints, UiContext,
};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use vn_scene::{BoxPrimitiveData, Color, Rect, Transform};
use vn_ui_definitions::{ChildElement, Component, ui_component, ElementId, StateToParams};
use vn_ui_macros::UiElement;

#[derive(Debug, Copy, Clone)]
pub enum ButtonAction {
    Clicked,
}

pub struct ButtonParams<State, Message> {
    pub background: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub corner_radius: f32,
    pub child: ChildElement<State, Message>,
    pub interaction: InteractionState,
    pub on_click: EventHandler<ButtonAction, Message>,
}

ui_component!(Button<ButtonParams<State, Msg>>);

impl<State, Message: Clone> Component for Button<State, Message> {
    type State = State;
    type Message = Message;
    type Params = ButtonParams<State, Message>;

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

        params
            .child
            .borrow_mut()
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
            params.child.borrow_mut().draw(
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
        let mut messages = params.child.borrow_mut().handle_event(ctx, state, event);

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
