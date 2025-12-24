use vn_vttrpg_window::graphics::GraphicsContext;
use vn_vttrpg_window::renderer::SceneRenderer;
use vn_vttrpg_window::resource_manager::ResourceManager;
use vn_vttrpg_window::input::InputState;
use vn_vttrpg_window::StateLogic;
use std::f32::consts::PI;
use std::sync::Arc;
use web_time::Instant;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;

pub struct MainLogic {
    pub resource_manager: Arc<ResourceManager>,
    pub input: InputState,
    application_start: Instant,
}

impl StateLogic<SceneRenderer> for MainLogic {
    async fn new_from_graphics_context(
        _graphics_context: &GraphicsContext,
        resource_manager: Arc<ResourceManager>,
    ) -> anyhow::Result<Self> {
        let diffuse_bytes = include_bytes!("vn_dk_white_square_better_n.png");
        resource_manager.load_texture_from_bytes("vn_dk_white_square", diffuse_bytes)?;

        Ok(Self {
            resource_manager,
            input: InputState::new(),
            application_start: Instant::now(),
        })
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        self.input.handle_key(event);

        use winit::keyboard::{KeyCode, PhysicalKey};
        match (event.physical_key, event.state.is_pressed()) {
            (PhysicalKey::Code(KeyCode::Escape), true) => event_loop.exit(),
            _ => {
                log::info!("Key: {:?} State: {:?}", event.physical_key, event.state);
            }
        }
    }

    fn render_target(&self) -> vn_vttrpg_window::scene::Scene {
        use vn_vttrpg_window::primitives::{
            BoxPrimitive, Color, ImagePrimitive, PrimitiveProperties, Rect, Transform,
        };
        let mut scene = vn_vttrpg_window::scene::Scene::new();
        scene.add_box(BoxPrimitive {
            common: PrimitiveProperties {
                transform: Transform {
                    translation: [200.0, 200.0],
                    rotation: self.application_start.elapsed().as_secs_f32() * 0.5 * PI,
                    scale: [1.0, 1.0],
                    origin: [0.5, 0.5],
                },
                clip_area: Rect::NO_CLIP,
            },
            size: [200.0, 150.0],
            color: Color::RED,
            border_color: Color::WHITE,
            border_thickness: 5.0,
            corner_radius: 10.0,
        });

        scene.add_image(ImagePrimitive {
            common: PrimitiveProperties {
                transform: Transform {
                    translation: [200.0, 200.0],
                    rotation: self.application_start.elapsed().as_secs_f32() * 0.5 * PI,
                    scale: [1.0, 1.0],
                    origin: [0.5, 0.5],
                },
                clip_area: Rect::NO_CLIP,
            },
            size: [100.0, 100.0],
            texture: vn_vttrpg_window::TextureDescriptor::Name("vn_dk_white_square".to_string()),
            tint: Color::WHITE,
        });

        scene
    }
}
