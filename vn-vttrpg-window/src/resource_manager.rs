use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;
use crate::graphics::WgpuContext;
use crate::texture::Texture;

pub struct ResourceManager {
    wgpu: Arc<WgpuContext>,
    textures: RefCell<HashMap<String, Arc<Texture>>>,
}

impl ResourceManager {
    pub fn new(wgpu: Arc<WgpuContext>) -> Self {
        Self {
            wgpu,
            textures: RefCell::new(HashMap::new()),
        }
    }

    pub fn load_texture_from_bytes(&self, name: &str, bytes: &[u8]) -> Result<Arc<Texture>, anyhow::Error> {
        {
            let textures = self.textures.borrow();
            if let Some(texture) = textures.get(name) {
                return Ok(texture.clone());
            }
        }

        let texture = Texture::from_bytes(
            &self.wgpu.device,
            &self.wgpu.queue,
            bytes,
            Some(name),
        )?;

        let texture = Arc::new(texture);
        let mut textures = self.textures.borrow_mut();
        textures.insert(name.to_string(), texture.clone());
        Ok(texture)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_texture_from_path(
        &self,
        name: &str,
        path: &std::path::Path,
    ) -> Result<Arc<Texture>, anyhow::Error> {
        {
            let textures = self.textures.borrow();
            if let Some(texture) = textures.get(name) {
                return Ok(texture.clone());
            }
        }

        let texture = Texture::from_file(
            &self.wgpu.device,
            &self.wgpu.queue,
            path,
            Some(name),
        )?;

        let texture = Arc::new(texture);
        let mut textures = self.textures.borrow_mut();
        textures.insert(name.to_string(), texture.clone());
        Ok(texture)
    }

    pub fn get_texture(&self, name: &str) -> Option<Arc<Texture>> {
        self.textures.borrow().get(name).cloned()
    }
}
