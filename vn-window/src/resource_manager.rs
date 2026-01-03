use crate::graphics::WgpuContext;
use crate::text::{Font, FontFaceTrueScale, TextRenderer};
use crate::texture::{Texture, TextureId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;
use ttf_parser::GlyphId;
use vn_utils::result::MonoResult;

type GlyphKey = (String, u32, u32);

/// Manages textures, fonts, and cached text rendering.
pub struct ResourceManager {
    wgpu: Rc<WgpuContext>,
    textures: RefCell<HashMap<TextureId, Rc<Texture>>>,
    fonts: RefCell<HashMap<String, Rc<Font>>>,
    fallback_font: Rc<Font>,
    text_renderer: RefCell<TextRenderer>,
    glyph_cache: RefCell<HashMap<GlyphKey, Glyph>>,
    unused_glyphs: RefCell<HashSet<GlyphKey>>,
}

use crate::text::Glyph;

impl fmt::Debug for ResourceManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceManager")
            .field(
                "textures",
                &format!("{} loaded", self.textures.borrow().len()),
            )
            .field("fonts", &format!("{} loaded", self.fonts.borrow().len()))
            .field(
                "glyph_cache",
                &format!("{} cached", self.glyph_cache.borrow().len()),
            )
            .finish_non_exhaustive()
    }
}

impl ResourceManager {
    pub fn new(wgpu: Rc<WgpuContext>, fallback_font: &[u8]) -> Self {
        let fallback_font = Rc::new(Font::new(fallback_font.to_vec()));

        Self {
            text_renderer: RefCell::new(TextRenderer::new(&wgpu.device)),
            wgpu,
            textures: RefCell::new(HashMap::new()),
            fonts: RefCell::new(HashMap::new()),
            glyph_cache: RefCell::new(HashMap::new()),
            unused_glyphs: RefCell::new(HashSet::new()),
            fallback_font,
        }
    }

    pub fn load_texture_from_bytes(&self, bytes: &[u8]) -> Result<Rc<Texture>, anyhow::Error> {
        let texture = Texture::from_bytes(&self.wgpu.device, &self.wgpu.queue, bytes)?;

        let texture = Rc::new(texture);
        let mut textures = self.textures.borrow_mut();
        textures.insert(texture.id.clone(), texture.clone());
        Ok(texture)
    }

    pub fn get_texture(&self, id: TextureId) -> Option<Rc<Texture>> {
        self.textures.borrow().get(&id).cloned()
    }

    pub fn load_font_from_bytes(
        &self,
        name: &str,
        bytes: &[u8],
    ) -> Result<Rc<Font>, anyhow::Error> {
        {
            let fonts = self.fonts.borrow();
            if let Some(font) = fonts.get(name) {
                return Ok(font.clone());
            }
        }

        let font = Rc::new(Font::new(bytes.to_vec()));
        let mut fonts = self.fonts.borrow_mut();
        fonts.insert(name.to_string(), font.clone());
        Ok(font)
    }

    pub fn get_font(&self, name: &str) -> Result<Rc<Font>, Rc<Font>> {
        let font = self.fonts.borrow().get(name).cloned();
        font.ok_or_else(|| self.fallback_font.clone())
    }

    pub fn line_height(&self, font_name: &str, font_size: f32) -> f32 {
        let font = self.get_font(font_name);

        if font.is_err() {
            log::warn!("Font {} not found, falling back to default", font_name);
        }

        let font = self.get_font(font_name).value();

        let face = match font.face() {
            Ok(f) => f,
            Err(e) => {
                log::error!("Failed to parse font: {}", e);
                return font_size;
            }
        };

        face.line_height(font_size)
    }

    pub fn get_glyphs(
        &self,
        graphics_context: &crate::graphics::GraphicsContext,
        text: &str,
        font_name: &str,
        font_size: f32,
    ) -> Vec<Glyph> {
        let font = self.get_font(font_name);

        if font.is_err() {
            log::warn!("Font {} not found, falling back to default", font_name);
        }

        let font = self.get_font(font_name).value();

        let face = match font.face() {
            Ok(f) => f,
            Err(e) => {
                log::error!("Failed to parse font: {}", e);
                return Vec::new();
            }
        };

        let mut glyphs = Vec::new();
        let font_ptr = Rc::as_ptr(&font.data) as usize;
        let font_id = format!("{:x}", font_ptr);

        for c in text.chars() {
            let glyph_id = face.glyph_index(c).unwrap_or(GlyphId(0));

            let key = (
                font_id.clone(),
                glyph_id.0 as u32,
                (font_size * 100.0) as u32,
            );

            if let Some(glyph) = self.glyph_cache.borrow().get(&key) {
                self.unused_glyphs.borrow_mut().remove(&key);
                glyphs.push(glyph.clone());
                continue;
            }

            match self.text_renderer.borrow_mut().render_glyph(
                graphics_context,
                &font,
                glyph_id,
                font_size,
            ) {
                Ok(glyph) => {
                    self.glyph_cache
                        .borrow_mut()
                        .insert(key.clone(), glyph.clone());
                    self.textures
                        .borrow_mut()
                        .insert(glyph.texture.id.clone(), glyph.texture.clone());
                    glyphs.push(glyph);
                }
                Err(e) => log::error!("Failed to render glyph {}: {}", c, e),
            }
        }

        glyphs
    }

    // TODO: better cleanup strategy
    pub fn cleanup_unused_text(&self) {
        let mut glyph_cache = self.glyph_cache.borrow_mut();
        let mut unused_glyphs = self.unused_glyphs.borrow_mut();

        glyph_cache.retain(|key, _| !unused_glyphs.contains(key));

        for unused_glyph in unused_glyphs.drain() {
            let glyph = glyph_cache.remove(&unused_glyph);
            if let Some(glyph) = glyph {
                self.textures.borrow_mut().remove(&glyph.texture.id);
            }
        }

        for key in glyph_cache.keys() {
            unused_glyphs.insert(key.clone());
        }
    }
}
