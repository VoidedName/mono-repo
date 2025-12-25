use crate::primitives::{BoxPrimitive, ImagePrimitive, TextPrimitive};

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

/// Represents the entire scene to be rendered, consisting of multiple layers.
#[derive(Debug, Clone, Default)]
pub struct Scene {
    pub layers: Vec<Layer>,
    active_layers: Vec<usize>,
}

impl Scene {
    /// Creates a new scene with a single initial layer.
    pub fn new() -> Self {
        let mut scene = Self::default();
        scene.push_layer_on_top();
        scene
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
