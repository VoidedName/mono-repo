use crate::{Card, CardParams, ConcreteSize, Element, ElementId, SizeConstraints, UiContext};
use vn_vttrpg_window::{Color, Scene};

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
        // ctx.event_manager.register_hitbox(
        //     self.ui_id,
        //     0,
        //     Rect {
        //         position: [origin.0, origin.1],
        //         size: [size.width, size.height],
        //     },
        // );
        // if let Some(parent) = ctx.parent_id {
        //     ctx.event_manager.set_parent(self.ui_id, parent);
        // }
        //
        // let old_parent = ctx.parent_id;
        // ctx.parent_id = Some(self.ui_id);
        self.child.draw(ctx, origin, size, scene);
        // ctx.parent_id = old_parent;
    }
}
