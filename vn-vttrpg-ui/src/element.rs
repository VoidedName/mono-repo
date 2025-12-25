use crate::{ConcreteSize, SizeConstraints};
use vn_vttrpg_window::Scene;

/// Represents a UI element that can be laid out and drawn.
pub trait Element {
    /// Determines the size of the element given the layout constraints.
    fn layout(&mut self, constraints: SizeConstraints) -> ConcreteSize;

    /// Call this method to draw the element at the specified origin with the given size into the scene.
    ///
    /// !!! IF YOU OVERWRITE THIS METHOD, DEBUG FEATURES WILL NOT WORK !!!
    fn draw(&mut self, origin: (f32, f32), size: ConcreteSize, scene: &mut Scene) {
        self.draw_impl(origin, size, scene);
        #[cfg(feature = "debug_outlines")]
        {
            use vn_vttrpg_window::BoxPrimitive;

            scene.add_box(
                BoxPrimitive::builder()
                    .transform(|t| t.translation([origin.0, origin.1]))
                    .size([size.width, size.height])
                    .border_color(vn_vttrpg_window::Color::RED.with_alpha(0.8))
                    .border_thickness(5.0)
                    .build(),
            )
        }
    }

    /// Draws the element at the specified origin with the given size into the scene.
    ///
    /// !!! DO NOT MANUALLY CALL THIS, CALL [draw](Self::draw) INSTEAD !!!
    fn draw_impl(&mut self, origin: (f32, f32), size: ConcreteSize, scene: &mut Scene);
}
