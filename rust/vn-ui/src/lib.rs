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

mod components;
mod element;
mod element_world;
mod event_manager;
mod interaction;
mod layouts;
mod sizes;
pub mod text;
mod utils;

pub use components::*;
pub use element::*;
pub use element_world::*;
pub use event_manager::*;
pub use interaction::*;
pub use layouts::*;
pub use sizes::*;
pub use vn_ui_animation::*;
pub use vn_ui_animation_macros::*;

pub use vn_scene::{Color, KeyCode, KeyEvent, Rect, Scene};

/// This keeps the UI agnostic to any specific graphics and resource management
pub trait TextMetrics {
    fn size_of_text(&self, text: &str, font: &str, font_size: f32) -> (f32, f32);
    fn line_height(&self, font: &str, font_size: f32) -> f32;
    fn get_glyphs(&self, text: &str, font: &str, font_size: f32) -> Vec<vn_scene::GlyphData>;
}

pub trait TextFieldCallbacks {
    fn text_layout_changed(&mut self, layout: &text::layout::TextLayout);
}

pub struct StateToParamsArgs<'a, State> {
    pub state: &'a State,
    pub id: ElementId,
    pub ctx: &'a UiContext,
}

pub type StateToParams<State, Params> = Box<dyn Fn(StateToParamsArgs<State>) -> Params>;
