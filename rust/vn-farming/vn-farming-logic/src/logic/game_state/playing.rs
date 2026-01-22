use crate::logic::PlatformHooks;
use crate::logic::game_state::{GameStateEx, StartMenu};
use crate::map::{Map, MapParams, TileMap};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::Rect;
use vn_ui::{Element, ElementWorld, EventManager};
use vn_wgpu_window::resource_manager::{ResourceManager, Sampling};
use vn_wgpu_window::{GraphicsContext, WgpuScene};
use winit::event::{ElementState, KeyEvent, MouseButton};

pub struct Playing {
    ui: RefCell<Box<dyn Element<State = StartMenu>>>,
    event_manager: Rc<RefCell<EventManager>>,
}

impl Playing {
    pub async fn new(
        platform: Rc<Box<dyn PlatformHooks>>,
        gc: Rc<GraphicsContext>,
        rm: Rc<ResourceManager>,
    ) -> anyhow::Result<Self> {
        let tile_map = platform
            .load_file("maps/test_tile_map.png".to_string())
            .await?;
        let tile_map = rm.load_texture_from_bytes(&tile_map, Sampling::Nearest)?;

        let mut world = ElementWorld::new();

        let tile_size = 32.0;
        let tile_count_x = 2;
        let tile_count_y = 2;
        let tile_size_x = 1.0 / tile_count_x as f32;
        let tile_size_y = 1.0 / tile_count_x as f32;

        let tiles = (0..tile_count_y)
            .flat_map(|y| {
                (0..tile_count_x).map(move |x| Rect {
                    position: [x as f32 * tile_size_x, y as f32 * tile_size_y],
                    size: [tile_size_x, tile_size_y],
                })
            })
            .collect::<Vec<_>>();

        let ui = Map::new(
            Box::new(move |_| MapParams {
                tile_map: TileMap {
                    texture_id: tile_map.id.clone(),
                    tile_locations: tiles.clone(),
                },
                tile_size: 32.0 * 2.0,
                map: vec![
                    vec![0, 1, 2, 3],
                    vec![1, 2, 3, 0],
                    vec![2, 3, 0, 1],
                    vec![3, 0, 1, 2],
                ],
            }),
            &mut world,
        );

        Ok(Self {
            ui: RefCell::new(Box::new(ui)),
            event_manager: Rc::new(RefCell::new(EventManager::new())),
        })
    }
}

impl GameStateEx for Playing {
    type Event = ();

    fn process_events(&mut self) -> Option<Self::Event> {
        todo!()
    }

    fn render_target(&self, size: (f32, f32)) -> WgpuScene {
        todo!()
    }

    fn handle_key(&mut self, event: &KeyEvent) {
        todo!()
    }

    fn handle_mouse_position(&mut self, x: f32, y: f32) {
        todo!()
    }

    fn handle_mouse_button(
        &mut self,
        mouse_position: (f32, f32),
        button: MouseButton,
        state: ElementState,
    ) {
        todo!()
    }
}
