mod events;
mod grid;
mod ui;

pub use events::EditorEvent;
pub use grid::{Grid, TilesetGrid};

use crate::logic::game_state::GameStateEx;
use crate::logic::{PlatformHooks, TextMetric};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use vn_scene::TextureId;
use vn_tilemap::{
    TileFitStrategy, TileMapLayerMapSpecification, TileMapLayerSpecification, TileMapSpecification,
};
use vn_ui::InteractionEventKind::MouseScroll;
use vn_ui::{
    DynamicSize, Element, ElementId, ElementSize, ElementWorld, EventManager,
    InputTextFieldController, InputTextFieldControllerExt, InteractionEventKind,
    ScrollAreaCallbacks, SimpleLayoutCache, SimpleScrollAreaCallbacks, SizeConstraints, Stack,
    UiContext,
};
use vn_wgpu_window::resource_manager::{ResourceManager, Sampling};
use vn_wgpu_window::{GraphicsContext, WgpuScene};
use web_time::Instant;
use winit::event::{ElementState, KeyEvent, MouseButton};

pub struct Editor {
    ui: RefCell<Box<dyn Element<State = Editor>>>,
    event_manager: Rc<RefCell<EventManager>>,
    map_spec: TileMapSpecification,
    selected_layer_index: usize,
    button_events: Rc<RefCell<Vec<(ElementId, EditorEvent)>>>,
    graphics_context: Rc<GraphicsContext>,
    resource_manager: Rc<ResourceManager>,
    platform: Rc<Box<dyn PlatformHooks>>,
    loaded_tilesets: HashMap<String, TextureId>,
    tileset_path_controller: Rc<RefCell<InputTextFieldController>>,
    tile_width_controller: Rc<RefCell<InputTextFieldController>>,
    tile_height_controller: Rc<RefCell<InputTextFieldController>>,
    tileset_cols_controller: Rc<RefCell<InputTextFieldController>>,
    tileset_rows_controller: Rc<RefCell<InputTextFieldController>>,
    tileset_path_input_id: ElementId,
    tile_width_input_id: ElementId,
    tile_height_input_id: ElementId,
    tileset_cols_input_id: ElementId,
    tileset_rows_input_id: ElementId,
    pub tileset_scroll_controller: Rc<RefCell<SimpleScrollAreaCallbacks>>,
}

impl Editor {
    pub async fn new(
        platform: Rc<Box<dyn PlatformHooks>>,
        gc: Rc<GraphicsContext>,
        rm: Rc<ResourceManager>,
    ) -> anyhow::Result<Self> {
        let mut world = ElementWorld::new();
        let tileset_path_input_id = world.next_id();
        let tileset_path_controller = Rc::new(RefCell::new(InputTextFieldController::new(
            tileset_path_input_id,
        )));
        tileset_path_controller.borrow_mut().text = "[Base]BaseChip_pipo.png".to_string();

        let tile_width_input_id = world.next_id();
        let tile_width_controller = Rc::new(RefCell::new(InputTextFieldController::new(
            tile_width_input_id,
        )));

        let tile_height_input_id = world.next_id();
        let tile_height_controller = Rc::new(RefCell::new(InputTextFieldController::new(
            tile_height_input_id,
        )));

        let tileset_cols_input_id = world.next_id();
        let tileset_cols_controller = Rc::new(RefCell::new(InputTextFieldController::new(
            tileset_cols_input_id,
        )));

        let tileset_rows_input_id = world.next_id();
        let tileset_rows_controller = Rc::new(RefCell::new(InputTextFieldController::new(
            tileset_rows_input_id,
        )));

        let mut editor = Self {
            ui: RefCell::new(Box::new(Stack::new(vec![], &mut world))),
            event_manager: Rc::new(RefCell::new(EventManager::new())),
            map_spec: TileMapSpecification {
                grid_dimensions: (32.0, 32.0),
                map_dimensions: (20, 15),
                layers: vec![],
            },
            selected_layer_index: 0,
            button_events: Rc::new(RefCell::new(Vec::new())),
            graphics_context: gc,
            resource_manager: rm,
            platform,
            loaded_tilesets: HashMap::new(),
            tileset_path_controller,
            tile_width_controller,
            tile_height_controller,
            tileset_cols_controller,
            tileset_rows_controller,
            tileset_path_input_id,
            tile_width_input_id,
            tile_height_input_id,
            tileset_cols_input_id,
            tileset_rows_input_id,
            tileset_scroll_controller: Rc::new(RefCell::new(SimpleScrollAreaCallbacks::new())),
        };

        editor.rebuild_ui();

        Ok(editor)
    }

    fn rebuild_ui(&mut self) {
        let mut world = ElementWorld::new();
        self.button_events.borrow_mut().clear();
        let metrics = Rc::new(TextMetric {
            rm: self.resource_manager.clone(),
            gc: self.graphics_context.clone(),
        });

        let editor_ui = ui::build_editor_ui(self, &mut world, metrics);
        self.tileset_path_input_id = editor_ui.tileset_path_input_id;
        self.tile_width_input_id = editor_ui.tile_width_input_id;
        self.tile_height_input_id = editor_ui.tile_height_input_id;
        self.tileset_cols_input_id = editor_ui.tileset_cols_input_id;
        self.tileset_rows_input_id = editor_ui.tileset_rows_input_id;

        if let Some(layer) = self.map_spec.layers.get(self.selected_layer_index) {
            self.tile_width_controller.borrow_mut().text = layer.tile_dimensions.0.to_string();
            self.tile_height_controller.borrow_mut().text = layer.tile_dimensions.1.to_string();
            self.tileset_cols_controller.borrow_mut().text =
                layer.tile_set_dimensions.0.to_string();
            self.tileset_rows_controller.borrow_mut().text =
                layer.tile_set_dimensions.1.to_string();
        }

        *self.ui.borrow_mut() = editor_ui.root;
    }

    fn handle_event(&mut self, event: EditorEvent) -> Option<EditorEvent> {
        match event.clone() {
            EditorEvent::ScrollTileset(delta_y) => {
                let mut borrow = self.tileset_scroll_controller.borrow_mut();
                borrow.scroll_y = borrow.scroll_y() - delta_y;
            }
            EditorEvent::AddLayer => {
                let (w, h) = self.map_spec.map_dimensions;
                let tile_set = if let Some(first_ts) = self.loaded_tilesets.keys().next() {
                    first_ts.clone()
                } else {
                    "".to_string()
                };
                self.map_spec.layers.push(TileMapLayerSpecification {
                    tile_set,
                    tile_set_dimensions: (1, 1),
                    tile_dimensions: (32, 32),
                    fit_strategy: TileFitStrategy::Stretch,
                    map: TileMapLayerMapSpecification {
                        tiles: vec![vec![None; w as usize]; h as usize],
                    },
                });
                self.selected_layer_index = self.map_spec.layers.len() - 1;
                self.rebuild_ui();
            }
            EditorEvent::RemoveLayer(index) => {
                if index < self.map_spec.layers.len() {
                    self.map_spec.layers.remove(index);
                    if self.selected_layer_index >= self.map_spec.layers.len()
                        && !self.map_spec.layers.is_empty()
                    {
                        self.selected_layer_index = self.map_spec.layers.len() - 1;
                    }
                    self.rebuild_ui();
                }
            }
            EditorEvent::SelectLayer(index) => {
                if index < self.map_spec.layers.len() {
                    self.selected_layer_index = index;
                    self.rebuild_ui();
                }
            }
            EditorEvent::SaveMap => {
                log::info!("Save Map triggered (not implemented)");
                if let Ok(json) = serde_json::to_string_pretty(&self.map_spec) {
                    log::info!("Map JSON:\n{}", json);
                }
            }
            EditorEvent::LoadMap => {
                log::info!("Load Map triggered (not implemented)");
                // In a real app, this would open a file dialog
                // and then:
                // self.map_spec = serde_json::from_str(&json).unwrap();
                // self.rebuild_ui();
            }
            EditorEvent::OpenSettings => {
                log::info!("Open Settings triggered (not implemented)");
            }
            EditorEvent::ChangeMapDimensions(w, h) => {
                self.map_spec.map_dimensions = (w, h);
                // Resize all layers
                for layer in self.map_spec.layers.iter_mut() {
                    layer.map.tiles.resize(h as usize, vec![None; w as usize]);
                    for row in layer.map.tiles.iter_mut() {
                        row.resize(w as usize, None);
                    }
                }
                self.rebuild_ui();
            }
            EditorEvent::ChangeTileDimensions(w, h) => {
                if let Some(layer) = self.map_spec.layers.get_mut(self.selected_layer_index) {
                    layer.tile_dimensions = (w, h);
                }
                self.rebuild_ui();
            }
            EditorEvent::ChangeTileSetDimensions(w, h) => {
                if let Some(layer) = self.map_spec.layers.get_mut(self.selected_layer_index) {
                    layer.tile_set_dimensions = (w, h);
                }
                self.rebuild_ui();
            }
            EditorEvent::SelectTileset(tileset_path) => {
                log::info!("SelectTileset: path={}", tileset_path);
                let platform = self.platform.clone();
                let rm = self.resource_manager.clone();

                let result = pollster::block_on(platform.load_file(tileset_path.clone()));

                if let Ok(bytes) = result {
                    match rm.load_texture_from_bytes(&bytes, Sampling::Nearest) {
                        Ok(texture) => {
                            let texture_id = texture.id.clone();
                            if let Some(layer) =
                                self.map_spec.layers.get_mut(self.selected_layer_index)
                            {
                                layer.tile_set = tileset_path.clone();
                                layer.tile_set_dimensions = (
                                    texture.size.0 / layer.tile_dimensions.0,
                                    texture.size.1 / layer.tile_dimensions.1,
                                );
                                self.loaded_tilesets.insert(tileset_path, texture_id);
                                log::info!(
                                    "Loaded tileset: {} ({:?})",
                                    layer.tile_set,
                                    layer.tile_set_dimensions
                                );
                            }
                        }
                        Err(e) => log::error!("Failed to load texture: {}", e),
                    }
                } else if let Err(e) = result {
                    log::error!("Failed to load tileset file: {}", e);
                }
                self.rebuild_ui();
            }
            EditorEvent::LoadTilesetFromInput => {
                let path = self.tileset_path_controller.borrow().text.clone();
                if !path.is_empty() {
                    self.handle_event(EditorEvent::SelectTileset(path));
                }
            }
        }
        Some(event)
    }
}

impl GameStateEx for Editor {
    type Event = EditorEvent;

    fn process_events(&mut self) -> Option<Self::Event> {
        let events = self.event_manager.borrow_mut().process_events();
        let mut editor_event = None;
        for event in events.clone() {
            // currently always scrolling the tileset preview
            // need to change it in the future
            match &event.kind {
                MouseScroll { y } => {
                    self.handle_event(EditorEvent::ScrollTileset(*y));
                }
                _ => {}
            }

            if let Some(target) = event.target {

                if target == self.tileset_path_input_id {
                    match &event.kind {
                        InteractionEventKind::Keyboard(key_event) => {
                            self.tileset_path_controller
                                .borrow_mut()
                                .handle_key(key_event);
                        }
                        InteractionEventKind::Click { x, y, .. } => {
                            self.tileset_path_controller
                                .borrow_mut()
                                .handle_click(*x, *y);
                        }
                        _ => {}
                    }
                } else if target == self.tile_width_input_id {
                    match &event.kind {
                        InteractionEventKind::Keyboard(key_event) => {
                            let (val, changed) = {
                                let mut controller = self.tile_width_controller.borrow_mut();
                                controller.handle_key(key_event);
                                let val = controller.text.parse::<u32>().ok();
                                let changed = if let (Some(val), Some(layer)) =
                                    (val, self.map_spec.layers.get(self.selected_layer_index))
                                {
                                    val != layer.tile_dimensions.0
                                } else {
                                    false
                                };
                                (val, changed)
                            };

                            if changed {
                                if let (Some(val), Some(layer)) =
                                    (val, self.map_spec.layers.get(self.selected_layer_index))
                                {
                                    editor_event =
                                        self.handle_event(EditorEvent::ChangeTileDimensions(
                                            val,
                                            layer.tile_dimensions.1,
                                        ));
                                }
                            }
                        }
                        InteractionEventKind::Click { x, y, .. } => {
                            self.tile_width_controller.borrow_mut().handle_click(*x, *y);
                        }
                        _ => {}
                    }
                } else if target == self.tile_height_input_id {
                    match &event.kind {
                        InteractionEventKind::Keyboard(key_event) => {
                            let (val, changed) = {
                                let mut controller = self.tile_height_controller.borrow_mut();
                                controller.handle_key(key_event);
                                let val = controller.text.parse::<u32>().ok();
                                let changed = if let (Some(val), Some(layer)) =
                                    (val, self.map_spec.layers.get(self.selected_layer_index))
                                {
                                    val != layer.tile_dimensions.1
                                } else {
                                    false
                                };
                                (val, changed)
                            };

                            if changed {
                                if let (Some(val), Some(layer)) =
                                    (val, self.map_spec.layers.get(self.selected_layer_index))
                                {
                                    editor_event =
                                        self.handle_event(EditorEvent::ChangeTileDimensions(
                                            layer.tile_dimensions.0,
                                            val,
                                        ));
                                }
                            }
                        }
                        InteractionEventKind::Click { x, y, .. } => {
                            self.tile_height_controller
                                .borrow_mut()
                                .handle_click(*x, *y);
                        }
                        _ => {}
                    }
                } else if target == self.tileset_cols_input_id {
                    match &event.kind {
                        InteractionEventKind::Keyboard(key_event) => {
                            let (val, changed) = {
                                let mut controller = self.tileset_cols_controller.borrow_mut();
                                controller.handle_key(key_event);
                                let val = controller.text.parse::<u32>().ok();
                                let changed = if let (Some(val), Some(layer)) =
                                    (val, self.map_spec.layers.get(self.selected_layer_index))
                                {
                                    val != layer.tile_set_dimensions.0
                                } else {
                                    false
                                };
                                (val, changed)
                            };

                            if changed {
                                if let (Some(val), Some(layer)) =
                                    (val, self.map_spec.layers.get(self.selected_layer_index))
                                {
                                    editor_event =
                                        self.handle_event(EditorEvent::ChangeTileSetDimensions(
                                            val,
                                            layer.tile_set_dimensions.1,
                                        ));
                                }
                            }
                        }
                        InteractionEventKind::Click { x, y, .. } => {
                            self.tileset_cols_controller
                                .borrow_mut()
                                .handle_click(*x, *y);
                        }
                        _ => {}
                    }
                } else if target == self.tileset_rows_input_id {
                    match &event.kind {
                        InteractionEventKind::Keyboard(key_event) => {
                            let (val, changed) = {
                                let mut controller = self.tileset_rows_controller.borrow_mut();
                                controller.handle_key(key_event);
                                let val = controller.text.parse::<u32>().ok();
                                let changed = if let (Some(val), Some(layer)) =
                                    (val, self.map_spec.layers.get(self.selected_layer_index))
                                {
                                    val != layer.tile_set_dimensions.1
                                } else {
                                    false
                                };
                                (val, changed)
                            };

                            if changed {
                                if let (Some(val), Some(layer)) =
                                    (val, self.map_spec.layers.get(self.selected_layer_index))
                                {
                                    editor_event =
                                        self.handle_event(EditorEvent::ChangeTileSetDimensions(
                                            layer.tile_set_dimensions.0,
                                            val,
                                        ));
                                }
                            }
                        }
                        InteractionEventKind::Click { x, y, .. } => {
                            self.tileset_rows_controller
                                .borrow_mut()
                                .handle_click(*x, *y);
                        }
                        _ => {}
                    }
                }
            }
        }

        for event in events {
            if let Some(target) = event.target {
                let ev = self
                    .button_events
                    .borrow()
                    .iter()
                    .find(|(id, _)| *id == target)
                    .map(|(_, ev)| ev.clone());

                if let Some(ev) = ev {
                    if let InteractionEventKind::Click { .. } = event.kind {
                        editor_event = self.handle_event(ev);
                        break;
                    }
                }
            }
        }
        editor_event
    }

    fn render_target(&self, size: (f32, f32)) -> WgpuScene {
        // We need to rebuild the UI if the layers changed.
        // For now, let's just assume the UI is static, but in a real app
        // we might want to rebuild it or use a more dynamic approach.
        // Actually, let's try to rebuild the layer list part if needed.

        let mut scene = WgpuScene::new((size.0, size.1));

        let event_manager = self.event_manager.clone();
        event_manager.borrow_mut().clear_hitboxes();

        let mut ctx = UiContext {
            event_manager,
            parent_id: None,
            layout_cache: Box::new(SimpleLayoutCache::new()),
            interactive: true,
            clip_rect: vn_scene::Rect::NO_CLIP,
            now: Instant::now(),
        };

        self.ui.borrow_mut().layout(
            &mut ctx,
            self,
            SizeConstraints {
                min_size: ElementSize {
                    width: 0.0,
                    height: 0.0,
                },
                max_size: DynamicSize {
                    width: Some(size.0),
                    height: Some(size.1),
                },
                scene_size: (size.0, size.1),
            },
        );

        self.ui.borrow_mut().draw(
            &mut ctx,
            self,
            (0.0, 0.0),
            ElementSize {
                width: size.0,
                height: size.1,
            },
            &mut scene,
        );

        scene
    }

    fn handle_key(&mut self, event: &KeyEvent) {
        self.event_manager
            .borrow_mut()
            .queue_event(InteractionEventKind::Keyboard(event.clone()));
    }

    fn handle_mouse_position(&mut self, x: f32, y: f32) {
        self.event_manager
            .borrow_mut()
            .queue_event(InteractionEventKind::MouseMove { x, y });
    }

    fn handle_mouse_button(
        &mut self,
        mouse_position: (f32, f32),
        button: MouseButton,
        state: ElementState,
    ) {
        use vn_ui::MouseButton as UiMouseButton;
        let button = match button {
            MouseButton::Left => UiMouseButton::Left,
            MouseButton::Right => UiMouseButton::Right,
            MouseButton::Middle => UiMouseButton::Middle,
            _ => return,
        };

        let kind = match state {
            ElementState::Pressed => InteractionEventKind::MouseDown {
                button,
                x: mouse_position.0,
                y: mouse_position.1,
            },
            ElementState::Released => InteractionEventKind::MouseUp {
                button,
                x: mouse_position.0,
                y: mouse_position.1,
            },
        };
        self.event_manager.borrow_mut().queue_event(kind);
    }

    fn handle_mouse_wheel(&mut self, _delta_x: f32, delta_y: f32) {
        self.event_manager
            .borrow_mut()
            .queue_event(MouseScroll { y: delta_y })
    }
}
