use crate::utils::ToArray;
use crate::{ConcreteSize, Element, ElementId, SizeConstraints, UiContext};
use vn_vttrpg_window::{BoxPrimitive, Color, Rect, Scene};

pub struct ButtonParams {
    pub background: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub corner_radius: f32,
}

// A button is basically just a card... but it should support some animations
// (maybe cards should also support them? is that in general? how do i animate things?
// primitive properties would be simple enough, but the parent has no idea about any other props
// should i restructure things to take in parameter objects and then in the constructor take an
// animator that returns those properties? then we could just tick the entire ui and it would grab
// it from those animators... those animators could also listen to events? they'd be specific to a
// component)
pub struct Button {
    #[allow(dead_code)]
    id: ElementId,
    child: Box<dyn Element>,
    params: ButtonParams,
}

impl Button {
    pub fn new(child: Box<dyn Element>, params: ButtonParams, ctx: &mut UiContext) -> Self {
        Self {
            id: ctx.event_manager.next_id(),
            child,
            params,
        }
    }
}

impl Element for Button {
    fn id(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize {
        self.child.layout(ctx, constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ConcreteSize,
        scene: &mut Scene,
    ) {
        let is_hovered = ctx.event_manager.is_hovered(self.id);
        let is_focused = ctx.event_manager.is_focused(self.id);

        let mut background = self.params.background;
        let mut border_color = self.params.border_color;

        if is_hovered {
            background = background.lighten(0.1);
        }

        if is_focused {
            background = background.darken(0.1);
            border_color = Color::WHITE;
        }

        ctx.with_hitbox_hierarchy(
            self.id,
            scene.current_layer_id(),
            Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |ctx| {
                scene.add_box(
                    BoxPrimitive::builder()
                        .transform(|t| t.translation([origin.0, origin.1]))
                        .color(background)
                        .border_color(border_color)
                        .corner_radius(self.params.corner_radius)
                        .border_thickness(self.params.border_width)
                        .size([size.width, size.height])
                        .build(),
                );

                let margin = self.params.border_width * 2.0;
                self.child.draw(
                    ctx,
                    (
                        origin.0 + self.params.border_width,
                        origin.1 + self.params.border_width,
                    ),
                    ConcreteSize {
                        width: size.width.max(margin) - margin,
                        height: size.height.max(margin) - margin,
                    },
                    scene,
                );
            },
        );
    }
}
