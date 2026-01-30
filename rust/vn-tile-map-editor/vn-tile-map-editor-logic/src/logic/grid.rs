use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use vn_scene::{BoxPrimitiveData, Color, Rect, Scene, Transform};
use vn_ui::{
    DynamicDimension, DynamicSize, Element, ElementId, ElementImpl, ElementSize, ElementWorld,
    EventHandler, InteractionEvent, InteractionEventKind, MouseButton, SizeConstraints,
    StateToParams, StateToParamsArgs, UiContext, into_box_impl,
};

pub struct GridParams<State, Message: Clone> {
    pub rows: u32,
    pub cols: u32,
    pub grid_size: (f32, f32),
    pub grid_color: Color,
    pub grid_width: f32,
    pub event_handler: EventHandler<GridEvent, Message>,
    pub child: Box<
        dyn Fn(
                &ElementId,
                (u32, u32),
                &State,
                &UiContext,
            ) -> Option<Rc<RefCell<dyn Element<State = State, Message = Message>>>>
            + 'static,
    >,
}

pub struct GridState {
    pub mouse_over_cell: Option<(u32, u32)>,
    pub mouse_is_down: bool,
}

#[derive(Clone, Debug)]
pub enum GridEvent {
    MouseOverCell(u32, u32),
    MouseDown(MouseButton),
    MouseUp(MouseButton),
}

pub struct Grid<State: 'static, Message: Clone + 'static> {
    id: ElementId,
    params: StateToParams<State, GridParams<State, Message>>,
    offset: (f32, f32),
    layout: HashMap<(u32, u32), ElementSize>,
    _phantom: std::marker::PhantomData<Message>,
}

impl<State: 'static, Message: Clone + 'static> Grid<State, Message> {
    pub fn new<P: Into<StateToParams<State, GridParams<State, Message>>>>(
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Self {
        Self {
            id: world.borrow_mut().next_id(),
            params: params.into(),
            offset: (0.0, 0.0),
            layout: HashMap::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<State: 'static, Message: Clone + 'static> ElementImpl for Grid<State, Message> {
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
        let params = self.params.call(StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        let mut child_constraint = constraints;
        child_constraint.max_size = DynamicSize {
            width: DynamicDimension::Limit(params.grid_size.0),
            height: DynamicDimension::Limit(params.grid_size.1),
        };
        child_constraint.min_size = ElementSize::ZERO;

        for x in 0..=params.cols {
            for y in 0..params.rows {
                if let Some(child) = (params.child)(&self.id, (x, y), state, ctx) {
                    let size = child.borrow_mut().layout(ctx, state, child_constraint);
                    self.layout.insert((x, y), size);
                }
            }
        }

        ElementSize {
            width: params.grid_size.0 * params.cols as f32,
            height: params.grid_size.1 * params.rows as f32,
        }
        .clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut dyn Scene,
    ) {
        let params = self.params.call(StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        ctx.with_clipping(
            Rect {
                position: [origin.0, origin.1],
                size: [
                    params.cols as f32 * params.grid_size.0,
                    params.rows as f32 * params.grid_size.1,
                ],
            },
            |ctx| {
                ctx.with_hitbox_hierarchy(self.id, scene.current_layer_id(), ctx.clip_rect, |_| {});

                self.offset = (
                    ctx.clip_rect.position[0] - origin.0,
                    ctx.clip_rect.position[1] - origin.1,
                );
            },
        );

        for x in 0..=params.cols {
            let px = origin.0 + x as f32 * params.grid_size.0 - params.grid_width / 2.0;
            scene.add_box(BoxPrimitiveData {
                transform: Transform::builder().translation([px, origin.1]).build(),
                size: [
                    params.grid_width,
                    size.height.min(params.grid_size.1 * params.rows as f32),
                ],
                color: params.grid_color,
                border_radius: 0.0,
                border_color: Color::TRANSPARENT,
                border_thickness: 0.0,
                clip_rect: ctx.clip_rect,
            });
        }

        for y in 0..=params.rows {
            let px = origin.1 + y as f32 * params.grid_size.1 - params.grid_width / 2.0;
            scene.add_box(BoxPrimitiveData {
                transform: Transform::builder().translation([origin.0, px]).build(),
                size: [
                    size.width.min(params.grid_size.0 * params.cols as f32),
                    params.grid_width,
                ],
                color: params.grid_color,
                border_radius: 0.0,
                border_color: Color::TRANSPARENT,
                border_thickness: 0.0,
                clip_rect: ctx.clip_rect,
            });
        }

        for x in 0..=params.cols {
            for y in 0..params.rows {
                if let Some(child) = (params.child)(&self.id, (x, y), state, ctx) {
                    child.borrow_mut().draw(
                        ctx,
                        state,
                        (
                            origin.0 + x as f32 * params.grid_size.0,
                            origin.1 + y as f32 * params.grid_size.1,
                        ),
                        self.layout.get(&(x, y)).unwrap().clone(),
                        scene,
                    )
                }
            }
        }
    }

    fn handle_event_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        event: &InteractionEvent,
    ) -> Vec<Self::Message> {
        let params = self.params.call(StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        params
            .event_handler
            .handle(self.id, event, || match event.kind {
                InteractionEventKind::MouseDown {
                    local_y,
                    local_x,
                    button,
                    ..
                } => {
                    let x = ((local_x + self.offset.0) / params.grid_size.0) as u32;
                    let y = ((local_y + self.offset.1) / params.grid_size.1) as u32;
                    if (0..params.cols).contains(&x) && (0..params.rows).contains(&y) {
                        if Some(self.id) == event.target {
                            vec![GridEvent::MouseDown(button)]
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                }
                InteractionEventKind::MouseUp { button, .. } => vec![GridEvent::MouseUp(button)],
                InteractionEventKind::MouseMove {
                    local_x, local_y, ..
                } => {
                    if Some(self.id) == event.target {
                        let x = ((local_x + self.offset.0) / params.grid_size.0) as u32;
                        let y = ((local_y + self.offset.1) / params.grid_size.1) as u32;
                        if (0..params.cols).contains(&x) && (0..params.rows).contains(&y) {
                            vec![GridEvent::MouseOverCell(x, y)]
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                }
                _ => vec![],
            })
    }
}

into_box_impl!(Grid);
