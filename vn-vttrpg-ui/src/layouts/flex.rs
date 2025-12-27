use crate::{ConcreteSize, DynamicSize, Element, SizeConstraints, UiContext};
use vn_vttrpg_window::Scene;

#[derive(Clone, Copy)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Clone, Copy)]
pub struct FlexParams {
    pub direction: FlexDirection,
}

pub struct Flex {
    children: Vec<Box<dyn Element>>,
    layout: Vec<ConcreteSize>,
    params: FlexParams,
}

impl Flex {
    pub fn new(children: Vec<Box<dyn Element>>, params: FlexParams) -> Self {
        Self {
            layout: std::iter::repeat(ConcreteSize::ZERO)
                .take(children.len())
                .collect(),
            children,
            params,
        }
    }

    pub fn new_row(children: Vec<Box<dyn Element>>) -> Self {
        Self::new(
            children,
            FlexParams {
                direction: FlexDirection::Row,
            },
        )
    }

    pub fn new_column(children: Vec<Box<dyn Element>>) -> Self {
        Self::new(
            children,
            FlexParams {
                direction: FlexDirection::Column,
            },
        )
    }
}

// todo: allow for weight / spacing between children?
impl Element for Flex {
    fn layout(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize {
        // what do we do with containers that grow? like anchor?
        // do we extend constraints to denote that they should not grow along some axis?

        let mut total_in_direction: f32 = 0.0;
        let mut max_orthogonal: f32 = 0.0;

        let child_constraints = match self.params.direction {
            FlexDirection::Row => SizeConstraints {
                min_size: ConcreteSize {
                    width: 0.0,
                    height: constraints.min_size.height,
                },
                max_size: DynamicSize {
                    width: None,
                    height: constraints.max_size.height,
                },
                scene_size: constraints.scene_size,
            },
            FlexDirection::Column => SizeConstraints {
                min_size: ConcreteSize {
                    width: constraints.min_size.width,
                    height: 0.0,
                },
                max_size: DynamicSize {
                    width: constraints.max_size.width,
                    height: None,
                },
                scene_size: constraints.scene_size,
            },
        };

        for (idx, child) in self.children.iter_mut().enumerate() {
            let child_size = child.layout(ctx, child_constraints);

            match self.params.direction {
                FlexDirection::Row => {
                    total_in_direction += child_size.width;
                    max_orthogonal = max_orthogonal.max(child_size.height);
                }
                FlexDirection::Column => {
                    total_in_direction += child_size.height;
                    max_orthogonal = max_orthogonal.max(child_size.width);
                }
            }

            self.layout[idx] = child_size;
        }

        match self.params.direction {
            FlexDirection::Row => ConcreteSize {
                width: total_in_direction,
                height: max_orthogonal,
            },
            FlexDirection::Column => ConcreteSize {
                width: max_orthogonal,
                height: total_in_direction,
            },
        }
        .clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ConcreteSize,
        scene: &mut Scene,
    ) {
        let mut offset = match self.params.direction {
            FlexDirection::Row => origin.0,
            FlexDirection::Column => origin.1,
        };
        for (idx, child) in self.children.iter_mut().enumerate() {
            let mut child_size = self.layout[idx];

            match self.params.direction {
                FlexDirection::Row => {
                    // making sure we are not drawing out of bounds for some reason
                    child_size.width = child_size.width.min(size.width - (offset - origin.0));
                    child_size.height = child_size.height.min(size.height);

                    child.draw(ctx, (offset, origin.1), child_size, scene);
                    offset += self.layout[idx].width;
                }
                FlexDirection::Column => {
                    // making sure we are not drawing out of bounds for some reason
                    child_size.width = child_size.width.min(size.width);
                    child_size.height = child_size.height.min(size.height - (offset - origin.1));

                    child.draw(ctx, (origin.0, offset), child_size, scene);
                    offset += self.layout[idx].height;
                }
            }
        }
    }
}
