use vn_scene::{BoxPrimitiveData, ImagePrimitiveData, Layer, Scene, TextPrimitiveData};

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
        self.push_layer_on_top();
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

    pub fn add_box(&mut self, b: BoxPrimitiveData) {
        self.active_layer().add_box(b);
    }

    pub fn add_image(&mut self, i: ImagePrimitiveData) {
        self.active_layer().add_image(i);
    }

    pub fn add_text(&mut self, t: TextPrimitiveData) {
        self.active_layer().add_text(t);
    }
}

impl Scene for WgpuScene {
    fn add_box(&mut self, b: BoxPrimitiveData) {
        self.add_box(b);
    }

    fn add_image(&mut self, i: ImagePrimitiveData) {
        self.add_image(i);
    }

    fn add_text(&mut self, t: TextPrimitiveData) {
        self.add_text(t);
    }

    fn with_next_layer(&mut self, f: &mut dyn FnMut(&mut dyn Scene)) {
        self.push_layer();
        f(self);
        self.pop_layer();
    }

    fn with_top_layer(&mut self, f: &mut dyn FnMut(&mut dyn Scene)) {
        self.push_layer_on_top();
        f(self);
        self.pop_layer();
    }

    fn current_layer_id(&self) -> u32 {
        self.current_layer_id()
    }

    fn layers(&self) -> &[Layer] {
        &self.layers
    }

    fn extend(
        &mut self,
        other: &mut dyn Scene,
    ) {
        for layer in other.layers() {
            self.with_top_layer(|s| {
                for b in &layer.boxes {
                    Scene::add_box(s, b.clone());
                }
                for i in &layer.images {
                    Scene::add_image(s, i.clone());
                }
                for t in &layer.texts {
                    Scene::add_text(s, t.clone());
                }
            })
        }
    }
}