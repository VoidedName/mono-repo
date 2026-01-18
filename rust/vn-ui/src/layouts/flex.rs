use crate::{
    Element, ElementId, ElementImpl, ElementSize, SizeConstraints, StateToParams, UiContext,
};
use vn_scene::Scene;

#[derive(Clone, Copy)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Clone, Copy)]
pub struct FlexParams {
    pub direction: FlexDirection,
}

pub struct Flex<State> {
    id: ElementId,
    children: Vec<Box<dyn Element<State = State>>>,
    layout: Vec<ElementSize>,
    params: StateToParams<State, FlexParams>,
}

impl<State> Flex<State> {
    pub fn new(
        children: Vec<Box<dyn Element<State = State>>>,
        params: StateToParams<State, FlexParams>,
        ctx: &mut UiContext,
    ) -> Self {
        Self {
            id: ctx.event_manager.next_id(),
            layout: std::iter::repeat(ElementSize::ZERO)
                .take(children.len())
                .collect(),
            children,
            params,
        }
    }

    pub fn new_row(children: Vec<Box<dyn Element<State = State>>>, ctx: &mut UiContext) -> Self {
        Self::new(
            children,
            Box::new(|_, _| FlexParams {
                direction: FlexDirection::Row,
            }),
            ctx,
        )
    }

    pub fn new_column(children: Vec<Box<dyn Element<State = State>>>, ctx: &mut UiContext) -> Self {
        Self::new(
            children,
            Box::new(|_, _| FlexParams {
                direction: FlexDirection::Column,
            }),
            ctx,
        )
    }
}

// todo: allow for weight / spacing between children?
impl<State> ElementImpl for Flex<State> {
    type State = State;

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
        let mut total_in_direction: f32 = 0.0;
        let mut max_orthogonal: f32 = 0.0;
        let params = (self.params)(state, &ctx.now);

        let mut child_constraints = constraints.clone();

        for (_, child) in self.children.iter_mut().enumerate() {
            let child_size = child.layout_impl(ctx, state, child_constraints);

            match params.direction {
                FlexDirection::Row => {
                    max_orthogonal = max_orthogonal.max(child_size.height);
                }
                FlexDirection::Column => {
                    max_orthogonal = max_orthogonal.max(child_size.width);
                }
            }
        }

        match params.direction {
            FlexDirection::Row => {
                child_constraints.min_size.height = max_orthogonal;
                child_constraints.max_size.height = Some(max_orthogonal);
            }
            FlexDirection::Column => {
                child_constraints.min_size.width = max_orthogonal;
                child_constraints.max_size.width = Some(max_orthogonal);
            }
        }

        for (idx, child) in self.children.iter_mut().enumerate() {
            let child_size = child.layout_impl(ctx, state, child_constraints);

            match params.direction {
                FlexDirection::Row => {
                    total_in_direction += child_size.width;
                }
                FlexDirection::Column => {
                    total_in_direction += child_size.height;
                }
            }

            self.layout[idx] = child_size;
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
        let params = (self.params)(state, &ctx.now);

        let mut offset = match params.direction {
            FlexDirection::Row => origin.0,
            FlexDirection::Column => origin.1,
        };
        for (idx, child) in self.children.iter_mut().enumerate() {
            let mut child_size = self.layout[idx];

            match params.direction {
                FlexDirection::Row => {
                    // making sure we are not drawing out of bounds for some reason
                    child_size.width = child_size.width.min(size.width - (offset - origin.0));
                    child_size.height = child_size.height.min(size.height);

                    child.draw(ctx, state, (offset, origin.1), child_size, canvas);
                    offset += self.layout[idx].width;
                }
                FlexDirection::Column => {
                    // making sure we are not drawing out of bounds for some reason
                    child_size.width = child_size.width.min(size.width);
                    child_size.height = child_size.height.min(size.height - (offset - origin.1));

                    child.draw(ctx, state, (origin.0, offset), child_size, canvas);
                    offset += self.layout[idx].height;
                }
            }
        }
    }
}

