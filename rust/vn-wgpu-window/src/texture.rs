use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;
pub use vn_scene::TextureId;

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
