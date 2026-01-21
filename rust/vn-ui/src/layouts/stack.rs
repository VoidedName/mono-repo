use crate::{
    DynamicSize, Element, ElementId, ElementImpl, ElementSize, ElementWorld, Flex, SizeConstraints,
    UiContext,
};
use vn_scene::Scene;

pub struct Stack<State> {
    id: ElementId,
    children: Vec<Box<dyn Element<State = State>>>,
    children_size: Vec<ElementSize>,
}

impl<State> Stack<State> {
    pub fn new(children: Vec<Box<dyn Element<State = State>>>, world: &mut ElementWorld) -> Self {
        Stack {
            id: world.next_id(),
            children_size: vec![ElementSize::ZERO; children.len()],
            children,
        }
    }
}

impl<State> ElementImpl for Stack<State> {
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
        let mut max_width: f32 = 0.0;
        let mut max_height: f32 = 0.0;

        for (idx, child) in &mut self.children.iter_mut().enumerate() {
            let child_size = child.layout(ctx, state, constraints);

            max_width = max_width.max(child_size.width);
            max_height = max_height.max(child_size.height);

            self.children_size[idx] = child_size;
        }

        ElementSize {
            width: constraints.max_size.width.unwrap_or(max_width),
            height: constraints.max_size.height.unwrap_or(max_height),
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

        let mut draw_child = |child: &mut Box<dyn Element<State = State>>,
                              child_size: ElementSize,
                              canvas: &mut dyn Scene| {
            child.draw(
                ctx,
                state,
                (origin.0, origin.1),
                child_size.clamp_to_constraints(SizeConstraints {
                    min_size: ElementSize::ZERO,
                    max_size: DynamicSize {
                        width: Some(size.width),
                        height: Some(size.height),
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
}

pub trait StackExt: Element {
    fn stack_with(
        self,
        others: Vec<Box<dyn Element<State = Self::State>>>,
        world: &mut ElementWorld,
    ) -> Stack<Self::State>;
}

impl<E: Element + 'static> StackExt for E {
    fn stack_with(
        self,
        others: Vec<Box<dyn Element<State = Self::State>>>,
        world: &mut ElementWorld,
    ) -> Stack<Self::State> {
        let mut elements = others;
        elements.insert(0, Box::new(self));
        Stack::new(elements, world)
    }
}
