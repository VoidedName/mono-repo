use std::collections::HashMap;
use std::sync::Arc;
use crate::graphics::WgpuContext;
use crate::texture::Texture;

pub struct ResourceManager {
    wgpu: Arc<WgpuContext>,
    textures: HashMap<String, Arc<Texture>>,
}

impl ResourceManager {
    pub fn new(wgpu: Arc<WgpuContext>) -> Self {
        Self {
            wgpu,
            textures: HashMap::new(),
        }
    }

    pub fn load_texture(&mut self, name: &str, bytes: &[u8]) -> Result<Arc<Texture>, anyhow::Error> {
        if let Some(texture) = self.textures.get(name) {
            return Ok(texture.clone());
        }

        let texture = Texture::from_bytes(
            &self.wgpu.device,
            &self.wgpu.queue,
            bytes,
            Some(name),
        )?;

        let texture = Arc::new(texture);
        self.textures.insert(name.to_string(), texture.clone());
        Ok(texture)
    }

    pub fn get_texture(&self, name: &str) -> Option<Arc<Texture>> {
        self.textures.get(name).cloned()
    }
}
