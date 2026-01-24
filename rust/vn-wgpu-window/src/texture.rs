use crate::text::Glyph;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;
pub use vn_scene::TextureId;
use vn_utils::{TimedLRUCache};

/// Represents a loaded GPU texture with its view and sampler.
pub struct Texture {
    pub id: TextureId,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: (u32, u32),
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

// TODO: This is sort of hacky, ideally i would just pass around some "Texture World"
//  to derive ids from.
static TEXTURE_ID_MANAGER: Mutex<RefCell<TextureIdManager>> =
    Mutex::new(RefCell::new(TextureIdManager {
        free_ids: Vec::new(),
        next_id: 1,
    }));

#[derive(Debug)]
struct TextureIdManager {
    free_ids: Vec<u32>,
    next_id: u32,
}

fn next_texture_id() -> TextureId {
    let manager = TEXTURE_ID_MANAGER.lock().unwrap();
    let mut manager = manager.borrow_mut();

    if let Some(id) = manager.free_ids.pop() {
        return TextureId(Rc::new(id));
    }

    let id = manager.next_id;
    manager.next_id += 1;

    TextureId(Rc::new(id))
}

fn drop_textures(texture: &Texture) {
    let manager = TEXTURE_ID_MANAGER.lock().unwrap();
    let mut manager = manager.borrow_mut();

    manager.free_ids.push(*texture.id.0);
}

impl Drop for Texture {
    fn drop(&mut self) {
        drop_textures(self);
    }
}

impl Texture {
    pub fn next_id() -> TextureId {
        let id = next_texture_id();
        id
    }

    pub fn empty(
        device: &wgpu::Device,
        dimensions: (u32, u32),
        label: Option<&str>,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: dimensions.0.max(1),
            height: dimensions.1.max(1),
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: usage | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            id: Self::next_id(),
            texture,
            view,
            sampler,
            size: (dimensions.0.max(1), dimensions.1.max(1)),
        }
    }

    /// Loads a texture from raw bytes (supports various image formats).
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sampler: &wgpu::SamplerDescriptor,
        bytes: &[u8],
    ) -> anyhow::Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, sampler, &img)
    }

    /// Loads a texture from a file path.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_file(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sampler: &wgpu::SamplerDescriptor,
        path: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<Self> {
        let img = image::open(path)?;
        Self::from_image(device, queue, sampler, &img)
    }

    /// Loads a texture from a [`DynamicImage`].
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sampler: &wgpu::SamplerDescriptor,
        img: &image::DynamicImage,
    ) -> anyhow::Result<Self> {
        let rgba = img.to_rgba8();
        let dimensions = rgba.dimensions();

        Self::from_rgba(device, queue, &rgba, sampler, dimensions)
    }

    /// Loads a texture from raw RGBA pixel data.
    pub fn from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba: &[u8],
        sampler: &wgpu::SamplerDescriptor,
        dimensions: (u32, u32),
    ) -> anyhow::Result<Self> {
        let id = Self::next_id();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(format!("Texture {}", id).as_str()),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(sampler);

        Ok(Self {
            id,
            texture,
            view,
            sampler,
            size: dimensions,
        })
    }

    pub fn create_render_target(
        device: &wgpu::Device,
        dimensions: (u32, u32),
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: dimensions.0.max(1),
            height: dimensions.1.max(1),
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            id: Self::next_id(),
            texture,
            view,
            sampler,
            size: (dimensions.0.max(1), dimensions.1.max(1)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureAtlasKey {
    /// Font the Glyph is rendered in
    pub font_name: String,
    /// Id of the glyph in the font.
    pub glyph_id: u32,
    /// Font size the glyph is rendered in.
    pub glyph_size: u32,
}

pub struct TextureAtlas {
    pub texture: Rc<Texture>,
    current_x: u32,
    current_y: u32,
    row_height: u32,
    padding: u32,
}

impl std::fmt::Debug for TextureAtlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextureAtlas")
            .field("texture", &self.texture)
            .field("current_x", &self.current_x)
            .field("current_y", &self.current_y)
            .field("row_height", &self.row_height)
            .field("padding", &self.padding)
            .finish()
    }
}

impl TextureAtlas {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture = Texture::empty(
            device,
            (width, height),
            Some("Texture Atlas"),
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        );

        Self {
            texture: Rc::new(texture),
            current_x: 0,
            current_y: 0,
            row_height: 0,
            padding: 2,
        }
    }

    pub fn allocate(&mut self, width: u32, height: u32) -> Option<vn_scene::Rect> {
        if self.current_x + width + self.padding > self.texture.size.0 {
            self.current_x = 0;
            self.current_y += self.row_height + self.padding;
            self.row_height = 0;
        }

        if self.current_y + height + self.padding > self.texture.size.1 {
            return None;
        }

        let rect = vn_scene::Rect {
            position: [self.current_x as f32 / self.texture.size.0 as f32, self.current_y as f32 / self.texture.size.1 as f32],
            size: [width as f32 / self.texture.size.0 as f32, height as f32 / self.texture.size.1 as f32],
        };

        self.current_x += width + self.padding;
        self.row_height = self.row_height.max(height);

        Some(rect)
    }
}

// todo: repacking
pub struct TextureAtlasCatalog {
    pub atlases: Vec<TextureAtlas>,
    atlas_size: (u32, u32),
    cache: RefCell<TimedLRUCache<TextureAtlasKey, Glyph>>,
}

impl std::fmt::Debug for TextureAtlasCatalog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextureAtlasCatalog")
            .field("atlases", &self.atlases)
            .field("atlas_size", &self.atlas_size)
            .field("cache_size", &self.cache.borrow().len())
            .finish()
    }
}

impl TextureAtlasCatalog {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let initial_atlas = TextureAtlas::new(device, width, height);
        Self {
            atlases: vec![initial_atlas],
            atlas_size: (width, height),
            cache: RefCell::new(TimedLRUCache::new()),
        }
    }

    pub fn get_glyph(&self, key: &TextureAtlasKey) -> Option<Glyph> {
        self.cache.borrow_mut().get(key).cloned()
    }

    pub fn insert_glyph(&self, key: TextureAtlasKey, glyph: Glyph) {
        self.cache.borrow_mut().insert(key, glyph);
    }

    pub fn tick_cache(&self) {
        self.cache.borrow_mut().tick();
    }

    pub fn allocate(&mut self, device: &wgpu::Device, width: u32, height: u32) -> (vn_scene::Rect, Rc<Texture>) {
        if let Some(rect) = self.atlases.last_mut().unwrap().allocate(width, height) {
            return (rect, self.atlases.last().unwrap().texture.clone());
        }

        // Current atlas is full, add a new one
        let mut new_atlas = TextureAtlas::new(device, self.atlas_size.0, self.atlas_size.1);
        let rect = new_atlas.allocate(width, height).expect("Failed to allocate in a fresh atlas");
        let texture = new_atlas.texture.clone();
        self.atlases.push(new_atlas);

        (rect, texture)
    }
}
