use crate::{
    DynamicDimension, Element, ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints,
    StateToParams, UiContext,
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
    /// if true, all elements will be forced to the same size along the orthogonal axis.
    pub force_orthogonal_same_size: bool,
}

pub struct FlexChild<State: 'static, Message: 'static> {
    pub element: Box<dyn Element<State = State, Message = Message>>,
    pub weight: Option<f32>,
}

impl<State: 'static, Message: 'static> FlexChild<State, Message> {
    pub fn new(element: Box<dyn Element<State = State, Message = Message>>) -> Self {
        Self {
            element,
            weight: None,
        }
    }

    pub fn weighted(
        element: Box<dyn Element<State = State, Message = Message>>,
        weight: f32,
    ) -> Self {
        Self {
            element,
            weight: Some(weight),
        }
    }
}

pub trait WeightedElement<State, Message> {
    fn with_weight_element(self, weight: f32) -> FlexChild<State, Message>;
}

impl<State, Message, E: Element<State = State, Message = Message> + 'static>
    WeightedElement<State, Message> for E
{
    fn with_weight_element(self, weight: f32) -> FlexChild<State, Message> {
        FlexChild::weighted(Box::new(self), weight)
    }
}

impl<State, Message> WeightedElement<State, Message>
    for Box<dyn Element<State = State, Message = Message>>
{
    fn with_weight_element(self, weight: f32) -> FlexChild<State, Message> {
        FlexChild::weighted(self, weight)
    }
}

pub struct Flex<State: 'static, Message: 'static> {
    id: ElementId,
    children: Vec<FlexChild<State, Message>>,
    layout: Vec<ElementSize>,
    params: StateToParams<State, FlexParams>,
}

impl<State: 'static, Message: 'static> Flex<State, Message> {
    pub fn new<P: Into<StateToParams<State, FlexParams>>>(
        children: Vec<FlexChild<State, Message>>,
        params: P,
        world: &mut ElementWorld,
    ) -> Self {
        Self {
            id: world.next_id(),
            layout: std::iter::repeat(ElementSize::ZERO)
                .take(children.len())
                .collect(),
            children,
            params: params.into(),
        }
    }

    pub fn new_unweighted<P: Into<StateToParams<State, FlexParams>>>(
        children: Vec<Box<dyn Element<State = State, Message = Message>>>,
        params: P,
        world: &mut ElementWorld,
    ) -> Self {
        Self::new(
            children.into_iter().map(FlexChild::new).collect(),
            params,
            world,
        )
    }

    pub fn new_row(
        children: Vec<FlexChild<State, Message>>,
        force_orthogonal_same_size: bool,
        world: &mut ElementWorld,
    ) -> Self {
        let params = StateToParams(Box::new(move |_| FlexParams {
            direction: FlexDirection::Row,
            force_orthogonal_same_size,
        }));

        Self::new(children, params, world)
    }

    pub fn new_row_unweighted(
        children: Vec<Box<dyn Element<State = State, Message = Message>>>,
        force_orthogonal_same_size: bool,
        world: &mut ElementWorld,
    ) -> Self {
        let params = StateToParams(Box::new(move |_| FlexParams {
            direction: FlexDirection::Row,
            force_orthogonal_same_size,
        }));

        Self::new_unweighted(children, params, world)
    }

    pub fn new_column(
        children: Vec<FlexChild<State, Message>>,
        force_orthogonal_same_size: bool,
        world: &mut ElementWorld,
    ) -> Self {
        let params = StateToParams(Box::new(move |_| FlexParams {
            direction: FlexDirection::Column,
            force_orthogonal_same_size,
        }));

        Self::new(children, params, world)
    }

    pub fn new_column_unweighted(
        children: Vec<Box<dyn Element<State = State, Message = Message>>>,
        force_orthogonal_same_size: bool,
        world: &mut ElementWorld,
    ) -> Self {
        let params = StateToParams(Box::new(move |_| FlexParams {
            direction: FlexDirection::Column,
            force_orthogonal_same_size,
        }));

        Self::new_unweighted(children, params, world)
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
        let params = self.params.call(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        let mut child_constraints = constraints;
        child_constraints.min_size.width = 0.0;
        child_constraints.min_size.height = 0.0;
        child_constraints.max_size.width =
            DynamicDimension::Hint(constraints.max_size.width.value());
        child_constraints.max_size.height =
            DynamicDimension::Hint(constraints.max_size.height.value());

        let mut total_weight = None;

        for (idx, child) in self.children.iter_mut().enumerate() {
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

        for (idx, child) in self.children.iter_mut().enumerate() {
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

            for (idx, child) in self.children.iter_mut().enumerate() {
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
        let params = self.params.call(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

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

                    child
                        .element
                        .draw(ctx, state, (offset, origin.1), child_size, canvas);
                    offset += self.layout[idx].width;
                }
                FlexDirection::Column => {
                    // making sure we are not drawing out of bounds for some reason
                    child_size.width = child_size.width.min(size.width);
                    child_size.height = child_size.height.min(size.height - (offset - origin.1));

                    child
                        .element
                        .draw(ctx, state, (origin.0, offset), child_size, canvas);
                    offset += self.layout[idx].height;
                }
            }
        }
    }

    fn handle_event_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        let mut messages = Vec::new();
        for child in &mut self.children {
            messages.extend(child.element.handle_event(ctx, state, event));
        }
        messages
    }
}

pub trait FlexExt: Element {
    fn flex<M, P: Into<StateToParams<Self::State, FlexParams>>>(
        self,
        others: Vec<Box<dyn Element<State = Self::State, Message = M>>>,
        params: P,
        world: &mut ElementWorld,
    ) -> Flex<Self::State, M>
    where
        Self: Element<Message = M> + 'static;

    fn flex_row<M>(
        self,
        others: Vec<Box<dyn Element<State = Self::State, Message = M>>>,
        force_orthogonal_same_size: bool,
        world: &mut ElementWorld,
    ) -> Flex<Self::State, M>
    where
        Self: Element<Message = M> + 'static;

    fn flex_column<M>(
        self,
        others: Vec<Box<dyn Element<State = Self::State, Message = M>>>,
        force_orthogonal_same_size: bool,
        world: &mut ElementWorld,
    ) -> Flex<Self::State, M>
    where
        Self: Element<Message = M> + 'static;

    fn flex_weighted<M, P: Into<StateToParams<Self::State, FlexParams>>>(
        self,
        others: Vec<FlexChild<Self::State, M>>,
        params: P,
        world: &mut ElementWorld,
    ) -> Flex<Self::State, M>
    where
        Self: Element<Message = M> + 'static;

    fn with_weight<M>(self, weight: f32) -> FlexChild<Self::State, M>
    where
        Self: Sized + Element<Message = M> + 'static;

    fn without_weight<M>(self) -> FlexChild<Self::State, M>
    where
        Self: Sized + Element<Message = M> + 'static;
}

impl<E: Element + 'static> FlexExt for E {
    fn flex<M, P: Into<StateToParams<Self::State, FlexParams>>>(
        self,
        others: Vec<Box<dyn Element<State = Self::State, Message = M>>>,
        params: P,
        world: &mut ElementWorld,
    ) -> Flex<Self::State, M>
    where
        Self: Element<Message = M> + 'static,
    {
        let mut elements: Vec<FlexChild<Self::State, M>> =
            others.into_iter().map(FlexChild::new).collect();
        elements.insert(0, FlexChild::new(Box::new(self)));
        Flex::new(elements, params, world)
    }

    fn flex_row<M>(
        self,
        others: Vec<Box<dyn Element<State = Self::State, Message = M>>>,
        force_orthogonal_same_size: bool,
        world: &mut ElementWorld,
    ) -> Flex<Self::State, M>
    where
        Self: Element<Message = M> + 'static,
    {
        let mut elements: Vec<FlexChild<Self::State, M>> =
            others.into_iter().map(FlexChild::new).collect();
        elements.insert(0, FlexChild::new(Box::new(self)));
        Flex::new_row(elements, force_orthogonal_same_size, world)
    }

    fn flex_column<M>(
        self,
        others: Vec<Box<dyn Element<State = Self::State, Message = M>>>,
        force_orthogonal_same_size: bool,
        world: &mut ElementWorld,
    ) -> Flex<Self::State, M>
    where
        Self: Element<Message = M> + 'static,
    {
        let mut elements: Vec<FlexChild<Self::State, M>> =
            others.into_iter().map(FlexChild::new).collect();
        elements.insert(0, FlexChild::new(Box::new(self)));
        Flex::new_column(elements, force_orthogonal_same_size, world)
    }

    fn flex_weighted<M, P: Into<StateToParams<Self::State, FlexParams>>>(
        self,
        others: Vec<FlexChild<Self::State, M>>,
        params: P,
        world: &mut ElementWorld,
    ) -> Flex<Self::State, M>
    where
        Self: Element<Message = M> + 'static,
    {
        let mut elements = others;
        elements.insert(0, FlexChild::new(Box::new(self)));
        Flex::new(elements, params, world)
    }

    fn with_weight<M>(self, weight: f32) -> FlexChild<Self::State, M>
    where
        Self: Sized + Element<Message = M> + 'static,
    {
        FlexChild::weighted(Box::new(self), weight)
    }

    fn without_weight<M>(self) -> FlexChild<Self::State, M>
    where
        Self: Sized + Element<Message = M> + 'static,
    {
        FlexChild::new(Box::new(self))
    }
}
