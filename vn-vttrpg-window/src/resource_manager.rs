use crate::graphics::WgpuContext;
use crate::text::{Font, FontFaceTrueScale};
use crate::texture::Texture;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ttf_parser::GlyphId;

/// Manages textures, fonts, and cached text rendering.
pub struct ResourceManager {
    wgpu: Rc<WgpuContext>,
    textures: RefCell<HashMap<String, Rc<Texture>>>,
    fonts: RefCell<HashMap<String, Rc<Font>>>,
    text_renderer: RefCell<Option<crate::text::renderer::TextRenderer>>,
    glyph_cache: RefCell<HashMap<(String, u32, u32), Glyph>>,
}

use crate::text::Glyph;

impl ResourceManager {
    // TODO: (bugs) implement fallback fonts

    pub fn new(wgpu: Rc<WgpuContext>) -> Self {
        Self {
            wgpu,
            textures: RefCell::new(HashMap::new()),
            fonts: RefCell::new(HashMap::new()),
            text_renderer: RefCell::new(None),
            glyph_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn load_texture_from_bytes(
        &self,
        name: &str,
        bytes: &[u8],
    ) -> Result<Rc<Texture>, anyhow::Error> {
        {
            let textures = self.textures.borrow();
            if let Some(texture) = textures.get(name) {
                return Ok(texture.clone());
            }
        }

        let texture = Texture::from_bytes(&self.wgpu.device, &self.wgpu.queue, bytes, Some(name))?;

        let texture = Rc::new(texture);
        let mut textures = self.textures.borrow_mut();
        textures.insert(name.to_string(), texture.clone());
        Ok(texture)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_texture_from_path(
        &self,
        name: &str,
        path: &std::path::Path,
    ) -> Result<Rc<Texture>, anyhow::Error> {
        {
            let textures = self.textures.borrow();
            if let Some(texture) = textures.get(name) {
                return Ok(texture.clone());
            }
        }

        let texture = Texture::from_file(&self.wgpu.device, &self.wgpu.queue, path, Some(name))?;

        let texture = Rc::new(texture);
        let mut textures = self.textures.borrow_mut();
        textures.insert(name.to_string(), texture.clone());
        Ok(texture)
    }

    pub fn get_texture(&self, name: &str) -> Option<Rc<Texture>> {
        self.textures.borrow().get(name).cloned()
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

    pub fn get_font(&self, name: &str) -> Option<Rc<Font>> {
        self.fonts.borrow().get(name).cloned()
    }

    pub fn line_height(&self, font_name: &str, font_size: f32) -> f32 {
        let font = match self.get_font(font_name) {
            Some(f) => f,
            None => {
                log::error!("Font {} not found", font_name);
                return font_size;
            }
        };

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
        let font = match self.get_font(font_name) {
            Some(f) => f,
            None => {
                log::error!("Font {} not found", font_name);
                return Vec::new();
            }
        };

        let mut text_renderer = self.text_renderer.borrow_mut();
        if text_renderer.is_none() {
            *text_renderer = Some(crate::text::renderer::TextRenderer::new(&self.wgpu.device));
        }
        let renderer = text_renderer.as_mut().unwrap();

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
                glyphs.push(glyph.clone());
                continue;
            }

            match renderer.render_glyph(graphics_context, &font, glyph_id, font_size) {
                Ok(glyph) => {
                    self.glyph_cache.borrow_mut().insert(key, glyph.clone());
                    glyphs.push(glyph);
                }
                Err(e) => log::error!("Failed to render glyph {}: {}", c, e),
            }
        }

        glyphs
    }

    pub fn cleanup_unused_text(&self) {
        let mut glyph_cache = self.glyph_cache.borrow_mut();
        glyph_cache.retain(|_, glyph| Rc::strong_count(&glyph.texture) > 1);
    }
}
