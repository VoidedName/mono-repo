use crate::utils::ToArray;
use crate::{
    Element, ElementId, ElementImpl, ElementSize, ElementWorld, EventHandler, InteractionEventKind,
    InteractionState, SizeConstraints, StateToParams, UiContext,
};
use vn_scene::{BoxPrimitiveData, Color, Rect, Scene, Transform};

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

pub struct Button<State: 'static, Message: 'static> {
    id: ElementId,
    child: Box<dyn Element<State = State, Message = Message>>,
    params: StateToParams<State, ButtonParams<Message>>,
}

impl<State, Message> Button<State, Message> {
    pub fn new<P: Into<StateToParams<State, ButtonParams<Message>>>>(
        child: Box<dyn Element<State = State, Message = Message>>,
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

impl<State, Message: Clone> ElementImpl for Button<State, Message> {
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

        let background = params.background;
        let border_color = params.border_color;

        ctx.with_hitbox_hierarchy(
            self.id,
            canvas.current_layer_id(),
            Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |ctx| {
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
                    clip_rect: ctx.clip_rect,
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
            },
        );
    }

    fn handle_event_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        let mut messages = self.child.handle_event(ctx, state, event);

        if event.target == Some(self.id) {
            let params = self.params.call(crate::StateToParamsArgs {
                state,
                id: self.id,
                ctx,
            });
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

pub trait ButtonExt: Element {
    fn button<P: Into<StateToParams<Self::State, ButtonParams<Self::Message>>>>(
        self,
        params: P,
        world: &mut ElementWorld,
    ) -> Button<Self::State, Self::Message>;
}

impl<E: Element + 'static> ButtonExt for E {
    fn button<P: Into<StateToParams<Self::State, ButtonParams<Self::Message>>>>(
        self,
        params: P,
        world: &mut ElementWorld,
    ) -> Button<Self::State, Self::Message> {
        Button::new(Box::new(self), params, world)
    }
}
