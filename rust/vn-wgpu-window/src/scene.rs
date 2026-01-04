use crate::primitives::{BoxPrimitive, GlyphInstance, ImagePrimitive, TextPrimitive};
use vn_scene::{BoxPrimitiveData, ImagePrimitiveData, Scene, TextPrimitiveData};

/// A collection of primitives to be rendered together.
#[derive(Debug, Clone, Default)]
pub struct Layer {
    pub boxes: Vec<BoxPrimitive>,
    pub images: Vec<ImagePrimitive>,
    pub texts: Vec<TextPrimitive>,
}

impl Layer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_box(&mut self, b: BoxPrimitive) {
        self.boxes.push(b);
    }

    pub fn add_image(&mut self, i: ImagePrimitive) {
        self.images.push(i);
    }

    pub fn add_text(&mut self, t: TextPrimitive) {
        self.texts.push(t);
    }
}

pub type SceneSize = (f32, f32);

/// Represents the entire scene to be rendered, consisting of multiple layers.
#[derive(Debug, Clone)]
pub struct WgpuScene {
    layers: Vec<Layer>,
    active_layers: Vec<usize>,
    scene_size: SceneSize,
}

impl WgpuScene {
    /// Creates a new scene with a single initial layer.
    pub fn new(size: SceneSize) -> Self {
        let mut scene = Self {
            layers: vec![],
            active_layers: vec![],
            scene_size: size,
        };

        scene.push_layer_on_top();
        scene.scene_size = size;
        scene
    }

    pub fn scene_size(&self) -> SceneSize {
        self.scene_size
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    pub fn current_layer_id(&self) -> u32 {
        *self.active_layers.last().unwrap() as u32
    }

    fn push_layer_on_top(&mut self) {
        let index = self.layers.len();
        self.layers.push(Layer::new());
        self.active_layers.push(index);
    }

    fn push_layer(&mut self) {
        let next_layer = self.active_layers.last().unwrap() + 1;
        if next_layer >= self.layers.len() {
            self.push_layer_on_top();
        } else {
            self.active_layers.push(next_layer);
        }
    }

    fn pop_layer(&mut self) {
        self.active_layers.pop();
    }

    fn active_layer(&mut self) -> &mut Layer {
        let index = *self
            .active_layers
            .last()
            .expect("No active layer! Did you pop too many times?");
        &mut self.layers[index]
    }

    pub fn with_top_layer<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.push_layer();
        f(self);
        self.pop_layer();
    }

    pub fn with_next_layer<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.push_layer();
        f(self);
        self.pop_layer();
    }

    pub fn add_box(&mut self, b: BoxPrimitive) {
        self.active_layer().add_box(b);
    }

    pub fn add_image(&mut self, i: ImagePrimitive) {
        self.active_layer().add_image(i);
    }

    pub fn add_text(&mut self, t: TextPrimitive) {
        self.active_layer().add_text(t);
    }
}

impl Scene for WgpuScene {
    fn add_box(&mut self, b: BoxPrimitiveData) {
        self.add_box(BoxPrimitive {
            common: crate::primitives::PrimitiveProperties {
                transform: b.transform,
                clip_area: b.clip_rect,
            },
            size: b.size,
            color: b.color,
            border_color: b.border_color,
            border_thickness: b.border_thickness,
            corner_radius: b.border_radius,
        });
    }

    fn add_image(&mut self, i: ImagePrimitiveData) {
        self.add_image(ImagePrimitive {
            common: crate::primitives::PrimitiveProperties {
                transform: i.transform,
                clip_area: i.clip_rect,
            },
            size: i.size,
            texture: i.texture_id.clone(),
            tint: i.tint,
        });
    }

    fn add_text(&mut self, t: TextPrimitiveData) {
        self.add_text(TextPrimitive {
            common: crate::primitives::PrimitiveProperties {
                transform: t.transform,
                clip_area: t.clip_rect,
            },
            glyphs: t
                .glyphs
                .into_iter()
                .map(|g| GlyphInstance {
                    texture: g.texture_id.clone(),
                    position: g.position,
                    size: g.size,
                })
                .collect(),
            tint: t.tint,
        });
    }

    fn with_next_layer(&mut self, f: &mut dyn FnMut(&mut dyn Scene)) {
        self.push_layer();
        f(self);
        self.pop_layer();
    }

    fn current_layer_id(&self) -> u32 {
        self.current_layer_id()
    }
}
