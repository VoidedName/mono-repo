use crate::{
    DynamicDimension, DynamicSize, Element, ElementId, ElementImpl, ElementSize, ElementWorld,
    SizeConstraints, UiContext,
};
use vn_scene::Scene;

pub struct Stack<State, Message> {
    id: ElementId,
    children: Vec<Box<dyn Element<State = State, Message = Message>>>,
    children_size: Vec<ElementSize>,
}

impl<State, Message> Stack<State, Message> {
    pub fn new(
        children: Vec<Box<dyn Element<State = State, Message = Message>>>,
        world: &mut ElementWorld,
    ) -> Self {
        Stack {
            id: world.next_id(),
            children_size: vec![ElementSize::ZERO; children.len()],
            children,
        }
    }
}

impl<State, Message> ElementImpl for Stack<State, Message> {
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
        let mut max_width: f32 = 0.0;
        let mut max_height: f32 = 0.0;

        for (idx, child) in &mut self.children.iter_mut().enumerate() {
            let child_size = child.layout(ctx, state, constraints);

            max_width = max_width.max(child_size.width);
            max_height = max_height.max(child_size.height);

            self.children_size[idx] = child_size;
        }

        ElementSize {
            width: max_width,
            height: max_height,
        }
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
        let mut first_drawn = false;

        let mut draw_child = |child: &mut Box<dyn Element<State = State, Message = Message>>,
                              child_size: ElementSize,
                              canvas: &mut dyn Scene| {
            child.draw(
                ctx,
                state,
                (origin.0, origin.1),
                child_size.clamp_to_constraints(SizeConstraints {
                    min_size: ElementSize::ZERO,
                    max_size: DynamicSize {
                        width: DynamicDimension::Limit(size.width),
                        height: DynamicDimension::Limit(size.height),
                    },
                    scene_size: (size.width, size.height), // Approximation
                }),
                canvas,
            );
        };

        for (idx, child) in self.children.iter_mut().enumerate() {
            match first_drawn {
                true => canvas.with_next_layer(&mut |canvas| {
                    draw_child(child, self.children_size[idx], canvas)
                }),
                false => {
                    draw_child(child, self.children_size[idx], canvas);
                    first_drawn = true;
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
            messages.extend(child.handle_event(ctx, state, event));
        }
        messages
    }
}

pub trait StackExt: Element {
    fn stack_with(
        self,
        others: Vec<Box<dyn Element<State = Self::State, Message = Self::Message>>>,
        world: &mut ElementWorld,
    ) -> Stack<Self::State, Self::Message>;
}

impl<E: Element + 'static> StackExt for E {
    fn stack_with(
        self,
        others: Vec<Box<dyn Element<State = Self::State, Message = Self::Message>>>,
        world: &mut ElementWorld,
    ) -> Stack<Self::State, Self::Message> {
        let mut elements = others;
        elements.insert(0, Box::new(self));
        Stack::new(elements, world)
    }
}
