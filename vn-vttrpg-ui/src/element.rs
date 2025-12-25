use crate::{Size, SizeConstraints};
use vn_vttrpg_window::Scene;

/// Represents a UI element that can be laid out and drawn.
pub trait Element {
    /// Determines the size of the element given the layout constraints.
    fn layout(&mut self, constraints: SizeConstraints) -> Size;

    /// Draws the element at the specified origin with the given size into the scene.
    fn draw(&mut self, origin: (f32, f32), size: Size, scene: &mut Scene);
}
