use crate::{
    DynamicDimension, Element, ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints,
    StateToParams, StateToParamsArgs, UiContext, into_box_impl,
};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::{Rect, Scene};

#[derive(Clone, Copy)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Clone)]
pub struct FlexParams<State: 'static, Message: 'static> {
    pub direction: FlexDirection,
    /// if true, all elements will be forced to the same size along the orthogonal axis.
    pub force_orthogonal_same_size: bool,
    pub children: Vec<Rc<RefCell<FlexChild<State, Message>>>>,
}

pub struct FlexChild<State: 'static, Message: 'static> {
    pub element: Box<dyn Element<State = State, Message = Message>>,
    pub weight: Option<f32>,
}

impl<State: 'static, Message: 'static> FlexChild<State, Message> {
    pub fn new(element: impl Into<Box<dyn Element<State = State, Message = Message>>>) -> Self {
        Self {
            element: element.into(),
            weight: None,
        }
    }

    pub fn weighted(
        element: impl Into<Box<dyn Element<State = State, Message = Message>>>,
        weight: f32,
    ) -> Self {
        Self {
            element: element.into(),
            weight: Some(weight),
        }
    }

    pub fn into_rc_refcell(self) -> Rc<RefCell<FlexChild<State, Message>>> {
        Rc::new(RefCell::new(self))
    }
}

impl<State, Message> Into<FlexChild<State, Message>>
    for Box<dyn Element<State = State, Message = Message>>
{
    fn into(self) -> FlexChild<State, Message> {
        FlexChild::new(self)
    }
}

pub struct Flex<State: 'static, Message: 'static> {
    id: ElementId,
    layout: Vec<ElementSize>,
    params: StateToParams<State, FlexParams<State, Message>>,
}

impl<State: 'static, Message: 'static> Flex<State, Message> {
    pub fn new<P: Into<StateToParams<State, FlexParams<State, Message>>>>(
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Self {
        Self {
            id: world.borrow_mut().next_id(),
            layout: Vec::new(),
            params: params.into(),
        }
    }
}

// todo: allow for weight / spacing between children?
impl<State, Message> ElementImpl for Flex<State, Message> {
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
        // what do we do with containers that grow? like anchor?
        // do we extend constraints to denote that they should not grow along some axis?
        let mut total_unweighted_in_direction: f32 = 0.0;
        let mut max_orthogonal: f32 = 0.0;
        let params = self.params.call(StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        if self.layout.len() < params.children.len() {
            self.layout.extend(vec![
                ElementSize::ZERO;
                params.children.len() - self.layout.len()
            ]);
        }

        let mut child_constraints = constraints;
        child_constraints.min_size.width = 0.0;
        child_constraints.min_size.height = 0.0;
        child_constraints.max_size.width =
            DynamicDimension::Hint(constraints.max_size.width.value());
        child_constraints.max_size.height =
            DynamicDimension::Hint(constraints.max_size.height.value());

        let mut total_weight = None;

        for (idx, child) in params.children.iter().enumerate() {
            let mut child = child.borrow_mut();
            let child_size = child.element.layout_impl(ctx, state, child_constraints);

            if let Some(weight) = child.weight {
                match total_weight {
                    None => total_weight = Some(weight),
                    Some(total) => total_weight = Some(total + weight),
                }
            }

            match params.direction {
                FlexDirection::Row => {
                    max_orthogonal = max_orthogonal.max(child_size.height);
                }
                FlexDirection::Column => {
                    max_orthogonal = max_orthogonal.max(child_size.width);
                }
            }

            self.layout[idx] = child_size;
        }

        match params.direction {
            FlexDirection::Row => {
                if params.force_orthogonal_same_size {
                    child_constraints.min_size.height = max_orthogonal;
                }
                child_constraints.max_size.height = DynamicDimension::Limit(max_orthogonal);
            }
            FlexDirection::Column => {
                if params.force_orthogonal_same_size {
                    child_constraints.min_size.width = max_orthogonal;
                }
                child_constraints.max_size.width = DynamicDimension::Limit(max_orthogonal);
            }
        }

        for (idx, child) in params.children.iter().enumerate() {
            let mut child = child.borrow_mut();
            if let Some(_) = child.weight {
                continue;
            }

            let child_size = child.element.layout_impl(ctx, state, child_constraints);

            match params.direction {
                FlexDirection::Row => {
                    total_unweighted_in_direction += child_size.width;
                }
                FlexDirection::Column => {
                    total_unweighted_in_direction += child_size.height;
                }
            }

            self.layout[idx] = child_size;
        }

        let remaining_available_space = match params.direction {
            FlexDirection::Row => constraints.max_size.width,
            FlexDirection::Column => constraints.max_size.height,
        }
        .map(|v| (v - total_unweighted_in_direction).max(0.0))
        .value();

        let mut total_in_direction = total_unweighted_in_direction;

        if let Some(total_weight) = total_weight {
            total_in_direction += remaining_available_space;

            let unit_per_weight = if total_weight > 0.0 {
                (remaining_available_space / total_weight).max(0.0)
            } else {
                0.0
            };

            for (idx, child) in params.children.iter().enumerate() {
                let mut child = child.borrow_mut();
                if child.weight.is_none() {
                    continue;
                }

                match params.direction {
                    FlexDirection::Row => {
                        let space = child.weight.unwrap() * unit_per_weight;
                        child_constraints.min_size.width = space;
                        child_constraints.max_size.width = DynamicDimension::Limit(space);
                    }
                    FlexDirection::Column => {
                        let space = child.weight.unwrap() * unit_per_weight;
                        child_constraints.min_size.height = space;
                        child_constraints.max_size.height = DynamicDimension::Limit(space);
                    }
                }

                self.layout[idx] = child.element.layout_impl(ctx, state, child_constraints);
            }
        }

        let size = match params.direction {
            FlexDirection::Row => ElementSize {
                width: total_in_direction,
                height: max_orthogonal,
            },
            FlexDirection::Column => ElementSize {
                width: max_orthogonal,
                height: total_in_direction,
            },
        }
        .clamp_to_constraints(constraints);

        size
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
    ) {
        let params = self.params.call(StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        ctx.with_clipping(
            Rect {
                position: [origin.0, origin.1],
                size: [size.width, size.height],
            },
            |ctx| {
                let mut offset = match params.direction {
                    FlexDirection::Row => origin.0,
                    FlexDirection::Column => origin.1,
                };
                for (idx, child) in params.children.iter().enumerate() {
                    let mut child = child.borrow_mut();
                    let mut child_size = self.layout[idx];

                    match params.direction {
                        FlexDirection::Row => {
                            // making sure we are not drawing out of bounds for some reason
                            child_size.width =
                                child_size.width.min(size.width - (offset - origin.0));
                            child_size.height = child_size.height.min(size.height);

                            child
                                .element
                                .draw(ctx, state, (offset, origin.1), child_size, canvas);
                            offset += self.layout[idx].width;
                        }
                        FlexDirection::Column => {
                            // making sure we are not drawing out of bounds for some reason
                            child_size.width = child_size.width.min(size.width);
                            child_size.height =
                                child_size.height.min(size.height - (offset - origin.1));

                            child
                                .element
                                .draw(ctx, state, (origin.0, offset), child_size, canvas);
                            offset += self.layout[idx].height;
                        }
                    }
                }
            },
        )
    }

    fn handle_event_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        let params = self.params.call(StateToParamsArgs {
            ctx,
            state,
            id: self.id,
        });

        let mut messages = Vec::new();
        for child in &mut params.children.iter() {
            messages.extend(child.borrow_mut().element.handle_event(ctx, state, event));
        }
        messages
    }
}

into_box_impl!(Flex);
