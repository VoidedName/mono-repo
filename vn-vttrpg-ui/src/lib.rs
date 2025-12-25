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

mod element;
mod layout;
mod sizes;

pub use element::*;
pub use layout::*;
pub use sizes::*;
use vn_vttrpg_window::{BoxPrimitive, Color, Scene, TextPrimitive};

pub struct Card {
    pub size: Size,
}

impl Element for Card {
    fn layout(&mut self, constraints: SizeConstraints) -> Size {
        self.size.clamp_to_constraints(constraints)
    }

    fn draw(&mut self, origin: (f32, f32), size: Size, scene: &mut Scene) {
        scene.add_box(
            BoxPrimitive::builder()
                .transform(|t| t.translation([origin.0, origin.1]))
                .size([size.width, size.height])
                .color(Color::WHITE)
                .build(),
        )
    }
}

/// A UI element that renders a string of text.
pub struct Label {
    pub text: String,
    /// Font name as registered in the resource manager.
    pub font: String,
    pub font_size: f32,
    pub size: Size,
    pub color: Color,
}

impl Element for Label {
    fn layout(&mut self, constraints: SizeConstraints) -> Size {
        self.size.clamp_to_constraints(constraints)
    }

    fn draw(&mut self, origin: (f32, f32), size: Size, scene: &mut Scene) {
        scene.add_text(
            TextPrimitive::builder(self.text.clone(), self.font.clone())
                .transform(|t| t.translation([origin.0, origin.1]))
                .size([size.width, size.height])
                .font_size(self.font_size)
                .tint(self.color)
                .build(),
        )
    }
}
