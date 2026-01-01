pub mod font;
pub mod renderer;

pub use font::Font;
pub use renderer::TextRenderer;

use crate::Texture;
use std::rc::Rc;

#[derive(Clone)]
pub struct Glyph {
    pub texture: Rc<Texture>,
    pub advance: f32,
    pub x_bearing: f32,
    pub y_offset: f32,
}
