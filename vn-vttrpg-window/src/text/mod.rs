pub mod font;
pub mod renderer;

pub use font::Font;
pub use renderer::TextRenderer;

use crate::Texture;
use std::sync::Arc;

#[derive(Clone)]
pub struct Glyph {
    pub texture: Arc<Texture>,
    pub advance: f32,
    pub y_offset: f32,
}
