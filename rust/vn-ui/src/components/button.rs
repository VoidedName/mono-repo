use crate::utils::ToArray;
use crate::{Element, ElementId, ElementImpl, ElementSize, SizeConstraints, UiContext};
use vn_scene::{BoxPrimitiveData, Color, Rect, Scene, Transform};

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

impl ElementImpl for Button {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        self.child
            .layout(ctx, constraints)
            .clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
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

        // We need a way to get the current layer ID without vn-wgpu-window.
        // Actually, we don't necessarily need it if register_hitbox doesn't use it,
        // but it does. Let's assume for now 0 is fine or we add it to the Canvas trait.
        // Wait, Canvas trait could have a method for this.

        ctx.with_hitbox_hierarchy(
            self.id,
            canvas.current_layer_id(),
            Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |ctx| {
                canvas.add_box(BoxPrimitiveData {
                    transform: Transform {
                        translation: [origin.0, origin.1],
                        ..Transform::DEFAULT
                    },
                    size: [size.width, size.height],
                    color: background,
                    border_color: border_color,
                    border_thickness: self.params.border_width,
                    border_radius: self.params.corner_radius,
                    clip_rect: Rect::NO_CLIP,
                });

                let margin = self.params.border_width * 2.0;
                self.child.draw(
                    ctx,
                    (
                        origin.0 + self.params.border_width,
                        origin.1 + self.params.border_width,
                    ),
                    ElementSize {
                        width: size.width.max(margin) - margin,
                        height: size.height.max(margin) - margin,
                    },
                    canvas,
                );
            },
        );
    }
}
