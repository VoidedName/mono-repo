use crate::utils::ToArray;
use crate::{Card, CardParams, ConcreteSize, Element, ElementId, SizeConstraints, UiContext};
use vn_vttrpg_window::{Color, Rect, Scene};

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
    ui_id: ElementId,
    child: Box<dyn Element>,
}

impl Button {
    pub fn new(child: Box<dyn Element>, params: ButtonParams, ctx: &mut UiContext) -> Self {
        let ui_id = ctx.event_manager.next_id();
        Self {
            ui_id,
            child: Box::new(Card::new(
                child,
                CardParams {
                    background_color: params.background,
                    border_color: params.border_color,
                    border_size: params.border_width,
                    corner_radius: params.corner_radius,
                },
            )),
        }
    }
}

impl Element for Button {
    fn layout(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize {
        self.child.layout(ctx, constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ConcreteSize,
        scene: &mut Scene,
    ) {
        ctx.with_hitbox_hierarchy(
            self.ui_id,
            scene.current_layer_id(),
            Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |ctx| {
                self.child.draw(ctx, origin, size, scene);
            },
        );
    }
}
