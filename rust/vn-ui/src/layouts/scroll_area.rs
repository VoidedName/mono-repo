use crate::utils::ToArray;
use crate::{
    DynamicDimension, Element, ElementId, ElementImpl, ElementSize, ElementWorld, ScrollAreaAction,
    SizeConstraints, StateToParams, UiContext,
};
use std::cell::RefCell;
use vn_scene::{BoxPrimitiveData, Color, Rect, Scene, Transform};

#[derive(Clone, Debug)]
pub struct ScrollAreaParams<Message: Clone> {
    pub scroll_x: Option<f32>,
    pub scroll_y: Option<f32>,
    pub on_scroll: Option<fn(ElementId, ScrollAreaAction) -> Message>,
    pub scrollbar_width: f32,
    pub scrollbar_margin: f32,
}

struct DragState {
    id: ElementId,
    initial_scroll: f32,
    initial_mouse: f32,
}

pub struct ScrollArea<State, Message: Clone> {
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
    pub fn new(
        child: Box<dyn Element<State = State, Message = Message>>,
        params: StateToParams<State, ScrollAreaParams<Message>>,
        world: &mut ElementWorld,
    ) -> Self {
        Self {
            id: world.next_id(),
            scroll_v_id: world.next_id(),
            scroll_h_id: world.next_id(),
            child,
            params,
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
        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        let mut child_constraints = constraints;
        let mut grow_by = ElementSize::ZERO;
        if params.scroll_x.is_some() {
            child_constraints.max_size.width = DynamicDimension::Hint(0.0);
            grow_by.height = params.scrollbar_width + params.scrollbar_margin;
        }
        if params.scroll_y.is_some() {
            child_constraints.max_size.height = DynamicDimension::Hint(0.0);
            grow_by.width = params.scrollbar_width + params.scrollbar_margin;
        }

        let child_size = self
            .child
            .layout(ctx, state, child_constraints.shrink_by(grow_by));

        self.child_size = child_size;

        let size = child_size
            .grow_by(grow_by)
            .clamp_to_constraints(constraints);

        self.viewport_size = size.shrink_by(grow_by);

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
        ctx.with_hitbox_hierarchy(
            self.id,
            scene.current_layer_id(),
            Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |ctx| {
                let params = (self.params)(crate::StateToParamsArgs {
                    state,
                    id: self.id,
                    ctx,
                });

                let child_origin = (
                    origin.0
                        - params
                            .scroll_x
                            .unwrap_or(0.0)
                            .min((self.child_size.width - size.width).max(0.0)),
                    origin.1
                        - params
                            .scroll_y
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
                if let Some(scroll_y) = params.scroll_y {
                    if self.child_size.height > size.height {
                        let scrollbar_height = (size.height / self.child_size.height) * size.height;
                        let scrollbar_y = (scroll_y / self.child_size.height) * size.height;

                        let scrollbar_rect = Rect {
                            position: [
                                origin.0 + size.width - params.scrollbar_width,
                                origin.1 + scrollbar_y,
                            ],
                            size: [params.scrollbar_width, scrollbar_height],
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
                            color: Color::WHITE.with_alpha(0.5),
                            border_color: Color::TRANSPARENT,
                            border_thickness: 0.0,
                            border_radius: params.scrollbar_width / 2.0,
                            clip_rect: Rect::NO_CLIP,
                        });
                    }
                }

                if let Some(scroll_x) = params.scroll_x {
                    if self.child_size.width > size.width {
                        let scrollbar_width = (size.width / self.child_size.width) * size.width;
                        let scrollbar_x = (scroll_x / self.child_size.width) * size.width;

                        let scrollbar_rect = Rect {
                            position: [
                                origin.0 + scrollbar_x,
                                origin.1 + size.height - params.scrollbar_width,
                            ],
                            size: [scrollbar_width, params.scrollbar_width],
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
                            color: Color::WHITE.with_alpha(0.5),
                            border_color: Color::TRANSPARENT,
                            border_thickness: 0.0,
                            border_radius: params.scrollbar_width / 2.0,
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
        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        let mut messages = Vec::new();

        match &event.kind {
            crate::InteractionEventKind::MouseDown { x, y, .. } => {
                if event.target == Some(self.scroll_v_id) {
                    *self.drag_state.borrow_mut() = Some(DragState {
                        id: self.scroll_v_id,
                        initial_scroll: params.scroll_y.unwrap_or(0.0),
                        initial_mouse: *y,
                    });
                } else if event.target == Some(self.scroll_h_id) {
                    *self.drag_state.borrow_mut() = Some(DragState {
                        id: self.scroll_h_id,
                        initial_scroll: params.scroll_x.unwrap_or(0.0),
                        initial_mouse: *x,
                    });
                }
            }
            crate::InteractionEventKind::MouseMove { x, y } => {
                if let Some(drag) = self.drag_state.borrow().as_ref() {
                    if let Some(on_scroll) = params.on_scroll {
                        if drag.id == self.scroll_v_id {
                            let delta_mouse = y - drag.initial_mouse;
                            let scroll_ratio = self.child_size.height / self.viewport_size.height;
                            let new_scroll = drag.initial_scroll + delta_mouse * scroll_ratio;
                            messages.push(on_scroll(self.id, ScrollAreaAction::ScrollY(new_scroll)));
                        } else if drag.id == self.scroll_h_id {
                            let delta_mouse = x - drag.initial_mouse;
                            let scroll_ratio = self.child_size.width / self.viewport_size.width;
                            let new_scroll = drag.initial_scroll + delta_mouse * scroll_ratio;
                            messages.push(on_scroll(self.id, ScrollAreaAction::ScrollX(new_scroll)));
                        }
                    }
                }
            }
            crate::InteractionEventKind::MouseUp { .. } => {
                *self.drag_state.borrow_mut() = None;
            }
            crate::InteractionEventKind::MouseScroll { y } => {
                if ctx.event_manager.borrow().is_hovered(self.id) {
                    if let Some(on_scroll) = params.on_scroll {
                        let current = params.scroll_y.unwrap_or(0.0);
                        messages.push(on_scroll(self.id, ScrollAreaAction::ScrollY(current - y)));
                    }
                }
            }
            _ => {}
        }

        messages.extend(self.child.handle_event(ctx, state, event));
        messages
    }
}

pub trait ScrollAreaExt: Element {
    fn scroll_area(
        self,
        params: StateToParams<Self::State, ScrollAreaParams<Self::Message>>,
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
    fn scroll_area(
        self,
        params: StateToParams<Self::State, ScrollAreaParams<Self::Message>>,
        world: &mut ElementWorld,
    ) -> ScrollArea<Self::State, Self::Message> {
        ScrollArea::new(Box::new(self), params, world)
    }
}
