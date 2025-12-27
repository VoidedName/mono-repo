// consider if elements should be stored as box<dyn> internally, or maybe arc/rc, or maybe just
// as some "id" which gets looked up in an element storage
// those would allow me to access the elements without traversing the tree, like updating an
// element directly (let's say the fps stat)
// or should the elements go and lookup data instead when being rendered? <-- probably this, avoids
// stale references and my state is free from pollution.
// example could be some thing DynamicText::new( Arc<Logic>, | Arc<Logic> | -> String )
//
// this does not solve ui restructuring, i.e. changing the tree (example, switching menus)
// can also be solved with callbacks from the ui elements (assuming they receive events)
// events don't have to be keyboard and click events, we could just feed any arbitrary event?
// in this case we could not pass the arc logic but rather just some event handler / listener?
// then a component can simply listen to it
//
// just receiving events, like mouse position, is not quite sufficient though, since with stacked
// elements, i would not know which one is the click target. This means at least a click / mouse event
// or mouse focus event would need to propagate through the tree to find the first valid target.
//
// what states do i need? should they be managed within the ui tree? externally? via callbacks?
//
// if allow absolute positioning, i.e. an element is placed independently of the constraints, then
// finding a mouse target is unreasonable. would i register their locations in a spacial index?

mod element;
mod event_manager;
mod layout;
mod sizes;

pub use element::*;
pub use event_manager::*;
pub use layout::*;
pub use sizes::*;
use vn_vttrpg_window::{Color, Rect, Scene, TextPrimitive};

/// This keeps the UI agnostic to any specific graphics and resource management
pub trait TextMetrics {
    fn size_of_text(&self, text: &str, font: &str, font_size: f32) -> (f32, f32);
}

#[derive(Clone)]
pub struct LabelParams {
    pub text: String,
    pub font: String,
    pub font_size: f32,
    pub color: Color,
}

/// A UI element that renders a string of text.
pub struct Label {
    params: LabelParams,
    size: ConcreteSize,
}

impl Label {
    pub fn new<T: TextMetrics>(params: LabelParams, text_metrics: &T) -> Self {
        let size = text_metrics.size_of_text(&params.text, &params.font, params.font_size);

        Self {
            params,
            size: ConcreteSize {
                width: size.0,
                height: size.1,
            },
        }
    }
}

impl Element for Label {
    fn layout(&mut self, _ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize {
        self.size.clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        _ctx: &mut UiContext,
        origin: (f32, f32),
        size: ConcreteSize,
        scene: &mut Scene,
    ) {
        scene.add_text(
            TextPrimitive::builder(self.params.text.clone(), self.params.font.clone())
                .transform(|t| t.translation([origin.0, origin.1]))
                // dunno if i should be squishing / stretching or clipping here...
                .size([self.size.width, self.size.height])
                .clip_area(|c| c.size([size.width, size.height]))
                .font_size(self.params.font_size)
                .tint(self.params.color)
                .build(),
        )
    }
}

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
