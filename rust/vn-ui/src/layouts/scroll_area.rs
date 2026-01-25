use crate::utils::ToArray;
use crate::{
    DynamicDimension, Element, ElementId, ElementImpl, ElementSize, ElementWorld, EventHandler,
    ScrollAreaAction, SizeConstraints, StateToParams, UiContext,
};
use std::cell::RefCell;
use vn_scene::{BoxPrimitiveData, Color, Rect, Scene, Transform};

#[derive(Copy, Clone, Debug)]
pub struct ScrollBarParams {
    pub position: Option<f32>,
    pub width: f32,
    pub margin: f32,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub struct ScrollAreaParams<Message> {
    pub scroll_x: ScrollBarParams,
    pub scroll_y: ScrollBarParams,
    pub scroll_action_handler: EventHandler<ScrollAreaAction, Message>,
}

struct DragState {
    id: ElementId,
    initial_scroll: f32,
    initial_mouse: f32,
}

pub struct ScrollArea<State: 'static, Message: 'static> {
    id: ElementId,
    scroll_v_id: ElementId,
    scroll_h_id: ElementId,
    child: Box<dyn Element<State = State, Message = Message>>,
    params: StateToParams<State, ScrollAreaParams<Message>>,
    child_size: ElementSize,
    viewport_size: ElementSize,
    drag_state: RefCell<Option<DragState>>,
}

impl<State, Message: Clone> ScrollArea<State, Message> {
    pub fn new<P: Into<StateToParams<State, ScrollAreaParams<Message>>>>(
        child: Box<dyn Element<State = State, Message = Message>>,
        params: P,
        world: &mut ElementWorld,
    ) -> Self {
        Self {
            id: world.next_id(),
            scroll_v_id: world.next_id(),
            scroll_h_id: world.next_id(),
            child,
            params: params.into(),
            child_size: ElementSize::ZERO,
            viewport_size: ElementSize::ZERO,
            drag_state: RefCell::new(None),
        }
    }
}

impl<State, Message: Clone> ElementImpl for ScrollArea<State, Message> {
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
        let mut grow_by = ElementSize::ZERO;

        child_constraints.max_size.width = DynamicDimension::Hint(0.0);
        grow_by.height = params.scroll_x.width + params.scroll_x.margin;

        child_constraints.max_size.height = DynamicDimension::Hint(0.0);
        grow_by.width = params.scroll_y.width + params.scroll_y.margin;

        let child_size = self
            .child
            .layout(ctx, state, child_constraints.shrink_by(grow_by));

        self.child_size = child_size;

        let size = child_size
            .grow_by(grow_by)
            .clamp_to_constraints(constraints);

        size
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut dyn Scene,
    ) {
        self.viewport_size = size;

        ctx.with_hitbox_hierarchy(
            self.id,
            scene.current_layer_id(),
            Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |ctx| {
                let params = self.params.call(crate::StateToParamsArgs {
                    state,
                    id: self.id,
                    ctx,
                });

                let child_origin = (
                    origin.0
                        - params
                            .scroll_x
                            .position
                            .unwrap_or(0.0)
                            .min((self.child_size.width - size.width).max(0.0)),
                    origin.1
                        - params
                            .scroll_y
                            .position
                            .unwrap_or(0.0)
                            .min((self.child_size.height - size.height).max(0.0)),
                );

                let clip_rect = Rect {
                    position: [origin.0, origin.1],
                    size: [size.width, size.height],
                };

                ctx.with_clipping(clip_rect, |ctx| {
                    self.child
                        .draw(ctx, state, child_origin, self.child_size, scene);
                });

                // Draw scroll bars
                {
                    if self.child_size.height > size.height {
                        let scrollbar_height = (size.height / self.child_size.height) * size.height;
                        let scrollbar_y = if let Some(scroll_y) = params.scroll_y.position {
                            (scroll_y / self.child_size.height) * size.height
                        } else {
                            0.0
                        };

                        let scrollbar_rect = Rect {
                            position: [
                                origin.0 + size.width - params.scroll_y.width,
                                origin.1 + scrollbar_y,
                            ],
                            size: [params.scroll_y.width, scrollbar_height],
                        };

                        ctx.with_hitbox_hierarchy(
                            self.scroll_v_id,
                            scene.current_layer_id(),
                            scrollbar_rect,
                            |_| {},
                        );

                        scene.add_box(BoxPrimitiveData {
                            transform: Transform {
                                translation: scrollbar_rect.position,
                                ..Transform::DEFAULT
                            },
                            size: scrollbar_rect.size,
                            color: params.scroll_y.color,
                            border_color: Color::TRANSPARENT,
                            border_thickness: 0.0,
                            border_radius: params.scroll_y.width / 2.0,
                            clip_rect: Rect::NO_CLIP,
                        });
                    }
                }

                {
                    if self.child_size.width > size.width {
                        let scrollbar_width = (size.width / self.child_size.width) * size.width;
                        let scrollbar_x = if let Some(scroll_x) = params.scroll_x.position {
                            (scroll_x / self.child_size.width) * size.width
                        } else {
                            0.0
                        };

                        let scrollbar_rect = Rect {
                            position: [
                                origin.0 + scrollbar_x,
                                origin.1 + size.height - params.scroll_x.width,
                            ],
                            size: [scrollbar_width, params.scroll_x.width],
                        };

                        ctx.with_hitbox_hierarchy(
                            self.scroll_h_id,
                            scene.current_layer_id(),
                            scrollbar_rect,
                            |_| {},
                        );

                        scene.add_box(BoxPrimitiveData {
                            transform: Transform {
                                translation: scrollbar_rect.position,
                                ..Transform::DEFAULT
                            },
                            size: scrollbar_rect.size,
                            color: params.scroll_x.color,
                            border_color: Color::TRANSPARENT,
                            border_thickness: 0.0,
                            border_radius: params.scroll_x.width / 2.0,
                            clip_rect: Rect::NO_CLIP,
                        });
                    }
                }
            },
        );
    }

    fn handle_event_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        let params = self.params.call(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        let mut messages =
            params
                .scroll_action_handler
                .handle(self.id, event, || match &event.kind {
                    crate::InteractionEventKind::MouseMove { x, y, .. } => {
                        if let Some(drag) = self.drag_state.borrow().as_ref() {
                            if drag.id == self.scroll_v_id {
                                let delta_mouse = y - drag.initial_mouse;
                                let scroll_ratio =
                                    self.child_size.height / self.viewport_size.height;
                                let new_scroll = drag.initial_scroll + delta_mouse * scroll_ratio;
                                return vec![ScrollAreaAction::ScrollY(new_scroll.clamp(
                                    0.0,
                                    self.child_size.height - self.viewport_size.height,
                                ))];
                            } else if drag.id == self.scroll_h_id {
                                let delta_mouse = x - drag.initial_mouse;
                                let scroll_ratio = self.child_size.width / self.viewport_size.width;
                                let new_scroll = drag.initial_scroll + delta_mouse * scroll_ratio;
                                return vec![ScrollAreaAction::ScrollX(new_scroll.clamp(
                                    0.0,
                                    self.child_size.height - self.viewport_size.height,
                                ))];
                            }
                        }
                        vec![]
                    }
                    crate::InteractionEventKind::MouseScroll { y } => {
                        if ctx.event_manager.borrow().is_hovered(self.id) {
                            let current = params.scroll_y.position.unwrap_or(0.0);
                            let next = (current - y)
                                .clamp(0.0, self.child_size.height - self.viewport_size.height);

                            if current != next {
                                return vec![ScrollAreaAction::ScrollY(next)];
                            }
                        }
                        vec![]
                    }
                    _ => vec![],
                });

        match &event.kind {
            crate::InteractionEventKind::MouseDown { x, y, .. } => {
                if event.target == Some(self.scroll_v_id) {
                    *self.drag_state.borrow_mut() = Some(DragState {
                        id: self.scroll_v_id,
                        initial_scroll: params.scroll_y.position.unwrap_or(0.0),
                        initial_mouse: *y,
                    });
                } else if event.target == Some(self.scroll_h_id) {
                    *self.drag_state.borrow_mut() = Some(DragState {
                        id: self.scroll_h_id,
                        initial_scroll: params.scroll_x.position.unwrap_or(0.0),
                        initial_mouse: *x,
                    });
                }
            }
            crate::InteractionEventKind::MouseUp { .. } => {
                *self.drag_state.borrow_mut() = None;
            }
            _ => {}
        }

        messages.extend(self.child.handle_event(ctx, state, event));
        messages
    }
}

pub trait ScrollAreaExt: Element {
    fn scroll_area<P: Into<StateToParams<Self::State, ScrollAreaParams<Self::Message>>>>(
        self,
        params: P,
        world: &mut ElementWorld,
    ) -> ScrollArea<Self::State, Self::Message>
    where
        Self: Sized + 'static,
        Self::Message: Clone;
}

impl<E: Element + 'static> ScrollAreaExt for E
where
    E::Message: Clone,
{
    fn scroll_area<P: Into<StateToParams<Self::State, ScrollAreaParams<Self::Message>>>>(
        self,
        params: P,
        world: &mut ElementWorld,
    ) -> ScrollArea<Self::State, Self::Message> {
        ScrollArea::new(Box::new(self), params, world)
    }
}
