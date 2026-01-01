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

pub trait FontFaceTrueScale {
    fn scale(&self, font_size: f32) -> f32;
    fn line_height(&self, font_size: f32) -> f32;
}

impl FontFaceTrueScale for ttf_parser::Face<'_> {
    /// The em box is not actually correct, as font_size / em_square will not give you the
    /// correct scale by which you have to scale the font points.
    /// Instead we need to use the ascender and descender to calculate the true scale.
    fn scale(&self, font_size: f32) -> f32 {
        // compensate for glyphs in reality exceeding the em box with negative descenders etc
        // making the actual true em square larger

        let units_per_em = self.units_per_em() as f32;
        let ascender = self.ascender() as f32;
        let descender = self.descender() as f32;

        let scale = (font_size / units_per_em) * ((ascender - descender) / units_per_em);

        scale
    }

    fn line_height(&self, font_size: f32) -> f32 {
        (self.ascender() - self.descender() + self.line_gap()) as f32 * self.scale(font_size)
    }
}
