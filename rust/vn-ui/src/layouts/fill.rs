use crate::{
    DynamicDimension, Element, ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints,
    UiContext, into_box_impl,
};

use vn_scene::Scene;

pub struct Fill<State, Message> {
    id: ElementId,
    element: Box<dyn Element<State = State, Message = Message>>,
}

impl<State, Message> Fill<State, Message> {
    pub fn new(
        element: impl Into<Box<dyn Element<State = State, Message = Message>>>,
        world: &mut ElementWorld,
    ) -> Self {
        Self {
            id: world.next_id(),
            element: element.into(),
        }
    }
}

impl<State, Message> ElementImpl for Fill<State, Message> {
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
        let child_size = self.element.layout(ctx, state, constraints);

        let height = match constraints.max_size.height {
            DynamicDimension::Limit(h) => h,
            DynamicDimension::Hint(_) => child_size.height,
        };

        let width = match constraints.max_size.width {
            DynamicDimension::Limit(w) => w,
            DynamicDimension::Hint(_) => child_size.width,
        };

        let mut desired_size = ElementSize { width, height }.clamp_to_constraints(constraints);

        if width > desired_size.width {
            let mut new_constraints = constraints;
            new_constraints.max_size.width = DynamicDimension::Limit(desired_size.width);
            let new_size = self.element.layout(ctx, state, new_constraints);
            desired_size = new_size.clamp_to_constraints(constraints);
        }

        desired_size
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
    ) {
        self.element.draw(ctx, state, origin, size, canvas);
    }

    fn handle_event_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        self.element.handle_event(ctx, state, event)
    }
}

pub trait FillExt<State, Message> {
    fn fill(self, world: &mut ElementWorld) -> Fill<State, Message>;
}

impl<State, Message, E: Into<Box<dyn Element<State = State, Message = Message>>> + 'static>
    FillExt<State, Message> for E
{
    fn fill(self, world: &mut ElementWorld) -> Fill<State, Message> {
        Fill::new(self, world)
    }
}

into_box_impl!(Fill);
