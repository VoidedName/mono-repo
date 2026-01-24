mod events;
mod grid;
mod ui;

pub use events::EditorEvent;
pub use grid::Grid;

use crate::logic::game_state::GameStateEx;
use crate::logic::{FpsStats, PlatformHooks, TextMetric};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use vn_scene::TextureId;
use vn_tilemap::{
    TileFitStrategy, TileMapLayerMapSpecification, TileMapLayerSpecification, TileMapSpecification,
};
use vn_ui::InteractionEventKind::MouseScroll;
use vn_ui::{
    DynamicDimension, DynamicSize, Element, ElementId, ElementSize, ElementWorld, EventManager,
    InteractionEventKind, SimpleLayoutCache, SizeConstraints, Stack, UiContext,
};
use vn_wgpu_window::resource_manager::{ResourceManager, Sampling};
use vn_wgpu_window::{GraphicsContext, WgpuScene};
use web_time::Instant;
use winit::event::{ElementState, KeyEvent, MouseButton};

pub struct Editor {
    ui: RefCell<Box<dyn Element<State = Editor, Message = EditorEvent>>>,
    event_manager: Rc<RefCell<EventManager>>,
    map_spec: TileMapSpecification,
    selected_layer_index: usize,
    button_events: Rc<RefCell<Vec<(ElementId, EditorEvent)>>>,
    graphics_context: Rc<GraphicsContext>,
    resource_manager: Rc<ResourceManager>,
    platform: Rc<Box<dyn PlatformHooks>>,
    loaded_tilesets: HashMap<String, TextureId>,
    tileset_path: String,
    tileset_path_caret: usize,
    tile_width_text: String,
    tile_width_caret: usize,
    tile_height_text: String,
    tile_height_caret: usize,
    tileset_cols_text: String,
    tileset_cols_caret: usize,
    tileset_rows_text: String,
    tileset_rows_caret: usize,
    tileset_path_input_id: ElementId,
    tile_width_input_id: ElementId,
    tile_height_input_id: ElementId,
    tileset_cols_input_id: ElementId,
    tileset_rows_input_id: ElementId,
    tileset_preview_scroll_area_id: ElementId,
    tileset_scroll_y: f32,
    tileset_scroll_x: f32,
    fps: Rc<RefCell<FpsStats>>,
}

impl Editor {
    pub async fn new(
        platform: Rc<Box<dyn PlatformHooks>>,
        gc: Rc<GraphicsContext>,
        rm: Rc<ResourceManager>,
        fps: Rc<RefCell<FpsStats>>,
    ) -> anyhow::Result<Self> {
        let mut world = ElementWorld::new();
        let tileset_path_input_id = world.next_id();
        let tile_width_input_id = world.next_id();
        let tile_height_input_id = world.next_id();
        let tileset_cols_input_id = world.next_id();
        let tileset_rows_input_id = world.next_id();

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
            tileset_path: "[Base]BaseChip_pipo.png".to_string(),
            tileset_path_caret: 0,
            tile_width_text: "".to_string(),
            tile_width_caret: 0,
            tile_height_text: "".to_string(),
            tile_height_caret: 0,
            tileset_cols_text: "".to_string(),
            tileset_cols_caret: 0,
            tileset_rows_text: "".to_string(),
            tileset_rows_caret: 0,
            tileset_path_input_id,
            tile_width_input_id,
            tile_height_input_id,
            tileset_cols_input_id,
            tileset_rows_input_id,
            tileset_preview_scroll_area_id: ElementId(0),
            tileset_scroll_y: 0.0,
            tileset_scroll_x: 0.0,
            fps,
        };

        editor.rebuild_ui();

        Ok(editor)
    }

    fn rebuild_ui(&mut self) {
        let old_focused = self.event_manager.borrow().focused_element();

        let mut world = ElementWorld::new();
        self.button_events.borrow_mut().clear();
        let metrics = Rc::new(TextMetric {
            rm: self.resource_manager.clone(),
            gc: self.graphics_context.clone(),
        });
        
        let editor_ui =
            ui::build_editor_ui(self, &mut world, self.resource_manager.clone(), metrics);

        let mut new_focused = None;
        if old_focused == Some(self.tileset_path_input_id) {
            new_focused = Some(editor_ui.tileset_path_input_id);
        } else if old_focused == Some(self.tile_width_input_id) {
            new_focused = Some(editor_ui.tile_width_input_id);
        } else if old_focused == Some(self.tile_height_input_id) {
            new_focused = Some(editor_ui.tile_height_input_id);
        } else if old_focused == Some(self.tileset_cols_input_id) {
            new_focused = Some(editor_ui.tileset_cols_input_id);
        } else if old_focused == Some(self.tileset_rows_input_id) {
            new_focused = Some(editor_ui.tileset_rows_input_id);
        }

        self.tileset_path_input_id = editor_ui.tileset_path_input_id;
        self.tile_width_input_id = editor_ui.tile_width_input_id;
        self.tile_height_input_id = editor_ui.tile_height_input_id;
        self.tileset_cols_input_id = editor_ui.tileset_cols_input_id;
        self.tileset_rows_input_id = editor_ui.tileset_rows_input_id;
        self.tileset_preview_scroll_area_id = editor_ui.tileset_preview_scroll_area_id;

        if let Some(f) = new_focused {
            self.event_manager.borrow_mut().set_focused_element(Some(f));
        }

        if let Some(layer) = self.map_spec.layers.get(self.selected_layer_index) {
            self.tile_width_text = layer.tile_dimensions.0.to_string();
            self.tile_height_text = layer.tile_dimensions.1.to_string();
            self.tileset_cols_text = layer.tile_set_dimensions.0.to_string();
            self.tileset_rows_text = layer.tile_set_dimensions.1.to_string();
        }

        *self.ui.borrow_mut() = editor_ui.root;
    }

    fn handle_event(&mut self, event: EditorEvent) {
        match event {
            EditorEvent::ScrollTileset(delta_y) => {
                self.tileset_scroll_y -= delta_y;
            }
            EditorEvent::ScrollAction { id, action } => {
                if id == self.tileset_preview_scroll_area_id {
                    match action {
                        vn_ui::ScrollAreaAction::ScrollX(x) => self.tileset_scroll_x = x,
                        vn_ui::ScrollAreaAction::ScrollY(y) => self.tileset_scroll_y = y,
                    }
                }
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
                let path = self.tileset_path.clone();
                if !path.is_empty() {
                    let ev = EditorEvent::SelectTileset(path);
                    self.handle_event(ev);
                }
            }
            EditorEvent::UpdateTilesetPath(text) => {
                self.tileset_path = text;
            }
            EditorEvent::UpdateTileWidth(text) => {
                let val = {
                    let is_valid = text.parse::<u32>().is_ok();
                    if is_valid {
                        self.tile_width_text = text;
                    } else if text.is_empty() {
                        self.tile_width_text = "0".to_string();
                    }
                    self.tile_width_text.parse::<u32>().ok()
                };
                if let Some(val) = val {
                    let old_val = self
                        .map_spec
                        .layers
                        .get(self.selected_layer_index)
                        .map(|l| l.tile_dimensions.0);
                    if let Some(old_val) = old_val {
                        if val != old_val {
                            let h = self.map_spec.layers[self.selected_layer_index]
                                .tile_dimensions
                                .1;
                            let ev = EditorEvent::ChangeTileDimensions(val, h);
                            self.handle_event(ev);
                        }
                    }
                }
            }
            EditorEvent::UpdateTileHeight(text) => {
                let val = {
                    let is_valid = text.parse::<u32>().is_ok();
                    if is_valid {
                        self.tile_height_text = text;
                    } else if text.is_empty() {
                        self.tile_height_text = "0".to_string();
                    }
                    self.tile_height_text.parse::<u32>().ok()
                };
                if let Some(val) = val {
                    let old_val = self
                        .map_spec
                        .layers
                        .get(self.selected_layer_index)
                        .map(|l| l.tile_dimensions.1);
                    if let Some(old_val) = old_val {
                        if val != old_val {
                            let w = self.map_spec.layers[self.selected_layer_index]
                                .tile_dimensions
                                .0;
                            let ev = EditorEvent::ChangeTileDimensions(w, val);
                            self.handle_event(ev);
                        }
                    }
                }
            }
            EditorEvent::UpdateTilesetCols(text) => {
                let val = {
                    let is_valid = text.parse::<u32>().is_ok();
                    if is_valid {
                        self.tileset_cols_text = text;
                    } else if text.is_empty() {
                        self.tileset_cols_text = "0".to_string();
                    }
                    self.tileset_cols_text.parse::<u32>().ok()
                };
                if let Some(val) = val {
                    let old_val = self
                        .map_spec
                        .layers
                        .get(self.selected_layer_index)
                        .map(|l| l.tile_set_dimensions.0);
                    if let Some(old_val) = old_val {
                        if val != old_val {
                            let h = self.map_spec.layers[self.selected_layer_index]
                                .tile_set_dimensions
                                .1;
                            let ev = EditorEvent::ChangeTileSetDimensions(val, h);
                            self.handle_event(ev);
                        }
                    }
                }
            }
            EditorEvent::UpdateTilesetRows(text) => {
                let val = {
                    let is_valid = text.parse::<u32>().is_ok();
                    if is_valid {
                        self.tileset_rows_text = text;
                    } else if text.is_empty() {
                        self.tileset_rows_text = "0".to_string();
                    }
                    self.tileset_rows_text.parse::<u32>().ok()
                };
                if let Some(val) = val {
                    let old_val = self
                        .map_spec
                        .layers
                        .get(self.selected_layer_index)
                        .map(|l| l.tile_set_dimensions.1);
                    if let Some(old_val) = old_val {
                        if val != old_val {
                            let w = self.map_spec.layers[self.selected_layer_index]
                                .tile_set_dimensions
                                .0;
                            let ev = EditorEvent::ChangeTileSetDimensions(w, val);
                            self.handle_event(ev);
                        }
                    }
                }
            }
            EditorEvent::TextFieldAction { id, action } => {
                use vn_ui::TextFieldAction::*;
                if id == self.tileset_path_input_id {
                    match action {
                        TextChange(text) => self.tileset_path = text,
                        CaretMove(caret) => self.tileset_path_caret = caret,
                    }
                } else if id == self.tile_width_input_id {
                    match action {
                        TextChange(text) => {
                            self.tile_width_text = text.clone();
                            self.handle_event(EditorEvent::UpdateTileWidth(text));
                        }
                        CaretMove(caret) => self.tile_width_caret = caret,
                    }
                } else if id == self.tile_height_input_id {
                    match action {
                        TextChange(text) => {
                            self.tile_height_text = text.clone();
                            self.handle_event(EditorEvent::UpdateTileHeight(text));
                        }
                        CaretMove(caret) => self.tile_height_caret = caret,
                    }
                } else if id == self.tileset_cols_input_id {
                    match action {
                        TextChange(text) => {
                            self.tileset_cols_text = text.clone();
                            self.handle_event(EditorEvent::UpdateTilesetCols(text));
                        }
                        CaretMove(caret) => self.tileset_cols_caret = caret,
                    }
                } else if id == self.tileset_rows_input_id {
                    match action {
                        TextChange(text) => {
                            self.tileset_rows_text = text.clone();
                            self.handle_event(EditorEvent::UpdateTilesetRows(text));
                        }
                        CaretMove(caret) => self.tileset_rows_caret = caret,
                    }
                }
            }
        }
    }
}

impl GameStateEx for Editor {
    type Event = EditorEvent;

    fn process_events(&mut self) -> Option<Self::Event> {
        let events = self.event_manager.borrow_mut().process_events();

        let mut ctx = UiContext {
            event_manager: self.event_manager.clone(),
            parent_id: None,
            layout_cache: Box::new(SimpleLayoutCache::new()),
            interactive: true,
            clip_rect: vn_scene::Rect::NO_CLIP,
            now: Instant::now(),
        };

        for event in &events {
            let messages = self.ui.borrow_mut().handle_event(&mut ctx, self, event);
            for msg in messages {
                self.handle_event(msg);
            }
        }

        None
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
                    width: DynamicDimension::Limit(size.0),
                    height: DynamicDimension::Limit(size.1),
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
            .queue_event(InteractionEventKind::MouseMove {
                x,
                y,
                local_x: x,
                local_y: y,
            });
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
                local_x: mouse_position.0,
                local_y: mouse_position.1,
            },
            ElementState::Released => InteractionEventKind::MouseUp {
                button,
                x: mouse_position.0,
                y: mouse_position.1,
                local_x: mouse_position.0,
                local_y: mouse_position.1,
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
