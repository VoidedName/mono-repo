use crate::{Size, SizeConstraints};
use vn_vttrpg_window::Scene;

pub trait Element {
    fn layout(&mut self, constraints: SizeConstraints) -> Size;
    fn draw(&mut self, origin: (f32, f32), size: Size, scene: &mut Scene);
}
