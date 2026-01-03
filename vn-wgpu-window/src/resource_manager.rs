use crate::graphics::WgpuContext;
use crate::text::{Font, FontFaceTrueScale, TextRenderer};
use crate::texture::{Texture, TextureId};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use ttf_parser::GlyphId;
use vn_utils::result::MonoResult;
use vn_utils::{TimedLRUCache, TimedLRUCacheCleanupParams};

type GlyphKey = (String, u32, u32);

/// Manages textures, fonts, and cached text rendering.
pub struct ResourceManager {
    wgpu: Rc<WgpuContext>,
    textures: RefCell<HashMap<TextureId, Rc<Texture>>>,
    fonts: RefCell<HashMap<String, Rc<Font>>>,
    fallback_font: Rc<Font>,
    text_renderer: RefCell<TextRenderer>,
    glyph_size_increment: Cell<f32>,
    glyph_cache: RefCell<TimedLRUCache<GlyphKey, Glyph>>,
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
            fallback_font,
            glyph_size_increment: Cell::new(4.0),
            glyph_cache: RefCell::new(TimedLRUCache::new()),
        }
    }

    pub fn set_glyph_size_increment(&self, increment: f32) {
        self.glyph_size_increment.set(increment);
    }

    pub fn load_texture_from_bytes(&self, bytes: &[u8]) -> Result<Rc<Texture>, anyhow::Error> {
        let texture = Texture::from_bytes(&self.wgpu.device, &self.wgpu.queue, bytes)?;

        let texture = Rc::new(texture);
        let mut textures = self.textures.borrow_mut();
        textures.insert(texture.id.clone(), texture.clone());
        Ok(texture)
    }

    pub fn add_texture(&self, texture: Rc<Texture>) {
        self.textures
            .borrow_mut()
            .insert(texture.id.clone(), texture);
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

        let increment = self.glyph_size_increment.get();
        let quantized_size = (font_size / increment).ceil() * increment;
        let scale_factor = font_size / quantized_size;

        for c in text.chars() {
            let glyph_id = face.glyph_index(c).unwrap_or(GlyphId(0));

            let key = (
                font_id.clone(),
                glyph_id.0 as u32,
                (quantized_size * 100.0) as u32,
            );

            if let Some(glyph) = self.glyph_cache.borrow_mut().get(&key) {
                let mut glyph = glyph.clone();
                glyph.size.0 *= scale_factor;
                glyph.size.1 *= scale_factor;
                glyph.advance *= scale_factor;
                glyph.x_bearing *= scale_factor;
                glyph.y_offset *= scale_factor;

                glyphs.push(glyph);
                continue;
            }

            match self.text_renderer.borrow_mut().render_glyph(
                graphics_context,
                self,
                &font,
                glyph_id,
                quantized_size,
            ) {
                Ok(mut glyph) => {
                    self.glyph_cache
                        .borrow_mut()
                        .insert(key.clone(), glyph.clone());

                    glyph.size.0 *= scale_factor;
                    glyph.size.1 *= scale_factor;
                    glyph.advance *= scale_factor;
                    glyph.x_bearing *= scale_factor;
                    glyph.y_offset *= scale_factor;

                    glyphs.push(glyph);
                }
                Err(e) => log::error!("Failed to render glyph {}: {}", c, e),
            }
        }

        glyphs
    }

    pub fn update(&self) {
        self.glyph_cache.borrow_mut().update();
    }

    pub fn cleanup(&self, max_age: u64, max_entries: usize) {
        let mut glyph_cache = self.glyph_cache.borrow_mut();

        // Prune glyph cache
        let _ = glyph_cache.cleanup(TimedLRUCacheCleanupParams {
            max_age: Some(max_age),
            max_entries: Some(max_entries),
        });

        // 2. Prune textures
        // probably a better way to do this... but works for now
        {
            let mut textures = self.textures.borrow_mut();
            textures.retain(|_, texture| {
                // Keep if someone else holds a strong reference to the Texture
                if Rc::strong_count(texture) > 1 {
                    return true;
                }

                // Keep if someone else holds a strong reference to the TextureId (e.g. a Glyph)
                // We expect 2 references internally: one in the cache and one in the Texture struct itself.
                if Rc::strong_count(&texture.id) > 2 {
                    return true;
                }

                false
            });
        }
    }
}
