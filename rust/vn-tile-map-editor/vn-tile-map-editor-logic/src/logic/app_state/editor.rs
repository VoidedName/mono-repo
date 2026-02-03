use crate::logic::app_state::editor_ui::{editor, layers, tileset};
use crate::logic::app_state::{
    ApplicationStateEx, LoadedTileSet, TryLoadTileSetResult, label, with_fps,
};
use crate::logic::{
    ApplicationContext, ApplicationEvent, EditorCallback, PlatformHooks, StateLogicDeferredEvent,
};
use crate::{UI_FONT, UI_FONT_SIZE};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use vn_scene::{CloneableScene, Color, ConstructableScene};
use vn_tilemap::{TileMapLayerMapSpecification, TileMapLayerSpecification, TileMapSpecification};
use vn_ui::{
    AnchorExt, AnchorLocation, AnchorParams, CardExt, CardParams, Conditional, ConditionalParams,
    Element, ElementWorld, Empty, EventManager, Flex, FlexChild, FlexDirection, FlexParams,
    PaddingExt, PaddingParams, ScrollBarParams, Stack, bottom, params,
};
use vn_wgpu_window::resource_manager::Sampling;

pub mod editor_ui;

#[derive(Debug)]
pub struct EditorState {
    loaded_tilesets: HashMap<String, LoadedTileSet>,
    current_layer: Option<usize>,
    tile_map: TileMapSpecification,
    tileset_view_scroll_x: ScrollBarParams,
    tileset_view_scroll_y: ScrollBarParams,
    tilemap_view_scroll_x: ScrollBarParams,
    tilemap_view_scroll_y: ScrollBarParams,
    layer_caret_positions: Vec<Option<usize>>,
    brush: Brush,
    errors: Vec<(String, web_time::Instant)>,
}

#[derive(Clone)]
pub struct Archive(pub Vec<u8>);

impl Debug for Archive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Archive({:?})", self.0.len())
    }
}

#[derive(Debug, Clone)]
pub enum EditorEvent {
    TilesetViewScrollX(f32),
    TilesetViewScrollY(f32),
    TilemapViewScrollX(f32),
    TilemapViewScrollY(f32),
    TryAddingLayer,
    TryLoadSpec,
    SaveSpec,
    AddLayer(TryLoadTileSetResult),
    SwitchToLayer(usize),
    DeleteLayer(usize),
    LayerCaretPosition(usize, usize),
    RenameLayer(usize, String),
    TileBrushSelect(Brush),
    Brushing(u32, u32),
    ChangeTilemapSize(u32, u32),
    HandleError(String),
    LoadSpecFromArchive(Archive),
    MoveLayer(usize, usize),
    LoadSpecFromData(TileMapSpecification, HashMap<String, LoadedTileSet>),
}

#[derive(Debug, Clone)]
pub enum Brush {
    Tileset((u32, u32), (u32, u32)),
    Eraser,
    None,
}

pub struct Editor<
    S: CloneableScene + ConstructableScene + 'static,
    Platform: PlatformHooks + 'static,
> {
    #[allow(unused)]
    ctx: ApplicationContext<S, Platform>,
    ui: RefCell<Box<dyn Element<State = EditorState, Message = EditorEvent>>>,
    state: EditorState,
    event_manager: Rc<RefCell<EventManager>>,
}

impl<S: CloneableScene + ConstructableScene, Platform: PlatformHooks + 'static>
    Editor<S, Platform>
{
    pub async fn new(ctx: ApplicationContext<S, Platform>) -> anyhow::Result<Self> {
        let world = Rc::new(RefCell::new(ElementWorld::new()));

        let title = label(
            |_| "Tile Map Editor".to_string(),
            UI_FONT,
            UI_FONT_SIZE,
            Color::WHITE,
            ctx.text_metrics.clone(),
            world.clone(),
        )
        .padding(params!(PaddingParams::vertical(25.0)), world.clone())
        .anchor(
            params!(AnchorParams {
                location: AnchorLocation::Top
            }),
            world.clone(),
        );

        let layers = layers(&ctx, world.clone());
        let editor = editor(&ctx, world.clone());
        let tileset = tileset(&ctx, world.clone());
        let error = label(
            |state: &EditorState| {
                state
                    .errors
                    .iter()
                    .map(|(e, _)| e.clone())
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            UI_FONT,
            UI_FONT_SIZE,
            Color::RED,
            ctx.text_metrics.clone(),
            world.clone(),
        )
        .padding(params!(PaddingParams::uniform(25.0)), world.clone())
        .card(
            params!(CardParams {
                border_color: Color::RED,
                border_size: 2.0,
                corner_radius: 5.0,
                background_color: Color::BLACK
            }),
            world.clone(),
        )
        .padding(params!(PaddingParams::uniform(25.0)), world.clone())
        .anchor(bottom!(), world.clone());

        let error = Conditional::new(
            error.into(),
            params!(
                args<EditorState>,
                ConditionalParams {
                    show: !args.state.errors.is_empty()
                }
            ),
            world.clone(),
        );

        let ui = Flex::new(
            {
                let children = vec![
                    FlexChild::new(title).into_rc_refcell(),
                    FlexChild::weighted(
                        Flex::new(
                            {
                                let children = vec![
                                    FlexChild::new(layers).into_rc_refcell(),
                                    FlexChild::new(Empty::new(params!(), world.clone()).padding(
                                        params!(PaddingParams::horizontal(25.0)),
                                        world.clone(),
                                    ))
                                    .into_rc_refcell(),
                                    FlexChild::weighted(editor, 1.0).into_rc_refcell(),
                                    FlexChild::new(Empty::new(params!(), world.clone()).padding(
                                        params!(PaddingParams::horizontal(25.0)),
                                        world.clone(),
                                    ))
                                    .into_rc_refcell(),
                                    FlexChild::new(tileset).into_rc_refcell(),
                                ];
                                params!(FlexParams {
                                    direction: FlexDirection::Row,
                                    force_orthogonal_same_size: true,
                                    children: children.clone(),
                                })
                            },
                            world.clone(),
                        ),
                        1.0,
                    )
                    .into_rc_refcell(),
                ];
                params!(FlexParams {
                    direction: FlexDirection::Column,
                    children: children.clone(),
                    force_orthogonal_same_size: true,
                })
            },
            world.clone(),
        );

        let ui = Stack::new(vec![Box::new(ui), Box::new(error)], world.clone());

        let scroll_bar = ScrollBarParams {
            width: 16.0,
            color: Color::WHITE,
            position: Some(0.0),
            margin: 8.0,
        };

        #[allow(unused_mut)]
        let mut s = Self {
            ui: RefCell::new(with_fps(&ctx, Box::new(ui), world.clone())),
            ctx,
            state: EditorState {
                brush: Brush::None,
                layer_caret_positions: Vec::new(),
                current_layer: None,
                loaded_tilesets: HashMap::new(),
                tile_map: TileMapSpecification {
                    version: 1,
                    layers: vec![],
                    map_dimensions: (10, 5),
                },
                tileset_view_scroll_x: scroll_bar,
                tileset_view_scroll_y: scroll_bar,
                tilemap_view_scroll_y: scroll_bar,
                tilemap_view_scroll_x: scroll_bar,
                errors: Vec::new(),
            },
            event_manager: Rc::new(RefCell::new(EventManager::new())),
        };

        #[cfg(feature = "example_map")]
        match s.load_example_map().await {
            Ok(()) => {}
            Err(e) => {
                s.handle_event(EditorEvent::HandleError(format!(
                    "Could not load example map: {}",
                    e
                )));
            }
        }

        Ok(s)
    }

    #[cfg(feature = "example_map")]
    async fn load_example_map(&mut self) -> anyhow::Result<()> {
        let archive = self
            .ctx
            .platform
            .load_asset("example_map.tar".to_string())
            .await?;

        self.handle_event(EditorEvent::LoadSpecFromArchive(Archive(archive)));

        Ok(())
    }
}

impl<S: CloneableScene + ConstructableScene + 'static, Platform: PlatformHooks + 'static>
    ApplicationStateEx for Editor<S, Platform>
{
    type StateEvent = EditorEvent;
    type State = EditorState;
    type ApplicationEvent = ApplicationEvent<S, Platform>;
    type Scene = S;

    fn ui(&self) -> &RefCell<Box<dyn Element<State = Self::State, Message = Self::StateEvent>>> {
        &self.ui
    }

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn event_manager(&self) -> Rc<RefCell<EventManager>> {
        self.event_manager.clone()
    }

    fn update(&mut self) {
        self.state
            .errors
            .retain(|(_, time)| time.elapsed().as_millis() < 5000);
    }

    fn handle_event(&mut self, event: Self::StateEvent) -> Option<Self::ApplicationEvent> {
        log::info!("handling state event: {:?}", event);

        match event {
            EditorEvent::MoveLayer(from, to) => {
                self.state.tile_map.layers.swap(from, to);
                self.state.layer_caret_positions.swap(from, to);
            }
            EditorEvent::HandleError(error) => {
                self.state.errors.push((error, web_time::Instant::now()));
            }
            EditorEvent::TileBrushSelect(brush) => self.state.brush = brush,
            EditorEvent::Brushing(x, y) => {
                if let Some(layer) = self.state.current_layer {
                    let layer = &mut self.state.tile_map.layers[layer];

                    match self.state.brush {
                        Brush::Tileset(from, to) => {
                            let mut x = x;
                            for b_x in from.0..=to.0 {
                                let mut y = y;
                                for b_y in from.1..=to.1 {
                                    if x < self.state.tile_map.map_dimensions.0
                                        && y < self.state.tile_map.map_dimensions.1
                                    {
                                        layer.map.tiles[y as usize][x as usize] =
                                            Some((b_x + b_y * layer.tileset_dimensions.0) as usize);
                                    }
                                    y += 1;
                                }
                                x += 1;
                            }
                        }
                        Brush::Eraser => {
                            layer.map.tiles[y as usize][x as usize] = None;
                        }
                        Brush::None => {}
                    }
                }
            }
            EditorEvent::DeleteLayer(layer) => {
                self.state.tile_map.layers.remove(layer);
                if let Some(current_layer) = self.state.current_layer {
                    if layer == current_layer {
                        self.state.brush = Brush::None;
                    }
                    if current_layer > 0 && current_layer >= layer {
                        self.state.current_layer = Some(current_layer - 1);
                    }
                    if self.state.tile_map.layers.len() == 0 {
                        self.state.current_layer = None;
                    }
                    self.state.layer_caret_positions.remove(layer);
                }
            }
            EditorEvent::SwitchToLayer(layer) => {
                self.state.brush = Brush::None;
                self.state.current_layer = Some(layer.clamp(0, self.state.tile_map.layers.len()));
            }
            EditorEvent::TilesetViewScrollX(v) => {
                self.state.tileset_view_scroll_x.position = Some(v)
            }
            EditorEvent::TilesetViewScrollY(v) => {
                self.state.tileset_view_scroll_y.position = Some(v)
            }
            EditorEvent::TilemapViewScrollX(v) => {
                self.state.tilemap_view_scroll_x.position = Some(v)
            }
            EditorEvent::TilemapViewScrollY(v) => {
                self.state.tilemap_view_scroll_y.position = Some(v)
            }
            EditorEvent::TryAddingLayer => {
                return Some(ApplicationEvent::NewLayer(
                    self.state.loaded_tilesets.keys().cloned().collect(),
                    EditorCallback {
                        call: Box::new(|editor, tiles| match tiles {
                            Some(tiles) => {
                                editor.handle_event(EditorEvent::AddLayer(tiles));
                            }
                            None => {}
                        }),
                    },
                ));
            }
            EditorEvent::AddLayer(tileset) => {
                match tileset {
                    TryLoadTileSetResult::Loaded(tileset) => {
                        let cols = tileset.texture_dimensions.0 / tileset.tile_dimensions.0;
                        let rows = tileset.texture_dimensions.1 / tileset.tile_dimensions.1;

                        let tiles = vec![
                            vec![None; self.state.tile_map.map_dimensions.0 as usize];
                            self.state.tile_map.map_dimensions.1 as usize
                        ];

                        self.state.tile_map.layers.push(TileMapLayerSpecification {
                            name: "".to_string(),
                            tileset: tileset.name.clone(),
                            tile_dimensions: (tileset.tile_dimensions.0, tileset.tile_dimensions.1),
                            map: TileMapLayerMapSpecification { tiles },
                            tileset_dimensions: (cols, rows),
                        });

                        self.state.layer_caret_positions.push(None);

                        self.state
                            .loaded_tilesets
                            .insert(tileset.name.clone(), tileset);
                    }
                    TryLoadTileSetResult::Reuse(tileset) => {
                        let tileset = self.state.loaded_tilesets.get(&tileset).unwrap();
                        self.handle_event(EditorEvent::AddLayer(TryLoadTileSetResult::Loaded(
                            tileset.clone(),
                        )));
                    }
                }
                self.state.current_layer = Some(self.state.tile_map.layers.len() - 1);
                self.state.brush = Brush::None;
            }
            EditorEvent::LayerCaretPosition(layer, caret) => {
                self.state.layer_caret_positions[layer] = Some(caret);
                self.handle_event(EditorEvent::SwitchToLayer(layer));
            }
            EditorEvent::RenameLayer(layer, name) => {
                self.state.tile_map.layers[layer].name = name;
            }
            EditorEvent::ChangeTilemapSize(cols, rows) => {
                self.state.tile_map.map_dimensions = (cols, rows);
                for layer in &mut self.state.tile_map.layers {
                    layer
                        .map
                        .tiles
                        .resize(rows as usize, vec![None; cols as usize]);
                    for row in &mut layer.map.tiles {
                        row.resize(cols as usize, None);
                    }
                }
            }
            EditorEvent::LoadSpecFromData(map, tilemaps) => {
                let bar = ScrollBarParams {
                    width: 16.0,
                    color: Color::WHITE,
                    position: Some(0.0),
                    margin: 8.0,
                };

                let current_layer = if map.layers.len() > 0 { Some(0) } else { None };

                self.state = EditorState {
                    loaded_tilesets: tilemaps,
                    current_layer,
                    layer_caret_positions: vec![None; map.layers.len()],
                    tile_map: map,
                    tileset_view_scroll_x: bar,
                    tileset_view_scroll_y: bar,
                    tilemap_view_scroll_x: bar,
                    tilemap_view_scroll_y: bar,
                    brush: Brush::None,
                    errors: self.state.errors.clone(),
                };
            }
            EditorEvent::LoadSpecFromArchive(data) => {
                let mut archive = tar::Archive::new(data.0.as_slice());
                let mut files = HashMap::new();

                match archive.entries() {
                    Ok(entries) => {
                        for entry in entries {
                            let mut entry = match entry {
                                Ok(entry) => entry,
                                Err(e) => {
                                    self.handle_event(EditorEvent::HandleError(format!(
                                        "Failed to get tar archive entry: {}",
                                        e
                                    )));
                                    return None;
                                }
                            };

                            let mut data = Vec::new();

                            match entry.read_to_end(&mut data) {
                                Ok(_) => {}
                                Err(e) => {
                                    self.handle_event(EditorEvent::HandleError(format!(
                                        "Failed to read tar archive entry: {}",
                                        e
                                    )));
                                    return None;
                                }
                            }

                            let path = match entry.path() {
                                Ok(path) => path,
                                Err(e) => {
                                    self.handle_event(EditorEvent::HandleError(format!(
                                        "Failed to get tar archive entry path: {}",
                                        e
                                    )));
                                    return None;
                                }
                            };

                            let name = match path.file_stem() {
                                Some(name) => name.to_string_lossy().to_string(),
                                None => {
                                    self.handle_event(EditorEvent::HandleError(
                                        "Tar archive entry has no file stem".to_string(),
                                    ));
                                    return None;
                                }
                            };

                            let extension =
                                path.extension().map(|e| e.to_string_lossy().to_string());

                            files.insert((name, extension), data);
                        }
                    }
                    Err(e) => {
                        self.handle_event(EditorEvent::HandleError(format!(
                            "Failed to get tar archive entries: {}",
                            e
                        )));
                        return None;
                    }
                };

                let mut loaded_tilesets = HashMap::new();
                let map = match files.get(&("tilemap".to_string(), Some("json".to_string()))) {
                    Some(map) => map,
                    None => {
                        self.handle_event(EditorEvent::HandleError(
                            "No tilemap found in archive".to_string(),
                        ));
                        return None;
                    }
                };

                let mut map: TileMapSpecification = match serde_json::from_slice(&map) {
                    Ok(map) => map,
                    Err(e) => {
                        self.handle_event(EditorEvent::HandleError(format!(
                            "Failed to parse tilemap: {}",
                            e
                        )));
                        return None;
                    }
                };

                for l in map.layers.iter_mut() {
                    let path = Path::new(&l.tileset);
                    let name = match path.file_stem() {
                        Some(name) => name.to_string_lossy().to_string(),
                        None => {
                            self.handle_event(EditorEvent::HandleError(
                                "Tileset has no file name".to_string(),
                            ));
                            return None;
                        }
                    };
                    let extension = path.extension().map(|e| e.to_string_lossy().to_string());

                    l.tileset = name.clone();

                    if !loaded_tilesets.contains_key(&name) {
                        let tileset = match files.remove(&(name.clone(), extension.clone())) {
                            Some(tileset) => tileset,
                            None => {
                                self.handle_event(EditorEvent::HandleError(format!(
                                    "Tileset {} not found in archive",
                                    name
                                )));
                                return None;
                            }
                        };

                        let tex = match self
                            .ctx
                            .rm
                            .load_texture_from_bytes(&tileset, Sampling::Nearest)
                        {
                            Ok(tex) => tex,
                            Err(_) => {
                                self.handle_event(EditorEvent::HandleError(format!(
                                    "Failed to load tileset {}",
                                    name
                                )));
                                return None;
                            }
                        };

                        loaded_tilesets.insert(
                            name.clone(),
                            LoadedTileSet {
                                name: name.clone(),
                                extension,
                                texture_id: tex.id.clone(),
                                texture_dimensions: (
                                    l.tileset_dimensions.0 * l.tile_dimensions.0,
                                    l.tileset_dimensions.1 * l.tile_dimensions.1,
                                ),
                                tile_dimensions: l.tile_dimensions,
                                bytes: Rc::new(RefCell::new(tileset)),
                            },
                        );
                    }
                }

                self.handle_event(EditorEvent::LoadSpecFromData(map, loaded_tilesets));
            }
            EditorEvent::TryLoadSpec => {
                let dispatcher = self.ctx.dispatcher.clone();
                let file = self.ctx.platform.pick_file(&["tar"]);
                self.ctx.platform.execute_async(async move {
                    if let Some(file) = file.await {
                        dispatcher.send_event(StateLogicDeferredEvent::Editor(
                            EditorEvent::LoadSpecFromArchive(Archive(file.bytes)),
                        ));
                    };
                });
            }
            EditorEvent::SaveSpec => {
                let tilesets = self
                    .state
                    .tile_map
                    .layers
                    .iter()
                    .filter_map(|l| self.state.loaded_tilesets.get(&l.tileset))
                    .collect::<Vec<_>>();

                let mut map = self.state.tile_map.clone();
                map.layers.iter_mut().for_each(|l| {
                    l.tileset = format!(
                        "{}{}",
                        l.tileset,
                        self.state
                            .loaded_tilesets
                            .get(&l.tileset)
                            .map(|ts| ts.extension.clone())
                            .flatten()
                            .map(|ext| format!(".{}", ext))
                            .unwrap_or_default()
                    );
                });

                let data = serde_json::to_vec(&map).unwrap();
                let mut archive = tar::Builder::new(Vec::new());
                let mut header = tar::Header::new_ustar();
                header.set_size(data.len() as u64);
                header.set_path("tilemap.json").unwrap();
                header.set_cksum();

                archive.append(&mut header, data.as_slice()).unwrap();

                let mut already_saved = HashSet::new();
                for ts in tilesets {
                    if already_saved.contains(&ts.file_name()) {
                        continue;
                    }
                    already_saved.insert(ts.file_name());

                    let data = ts.bytes.borrow();

                    let mut header = tar::Header::new_ustar();
                    header.set_size(data.len() as u64);
                    header.set_path(ts.file_name()).unwrap();
                    header.set_cksum();

                    archive.append(&mut header, data.as_slice()).unwrap();
                }

                match archive.finish() {
                    Ok(_) => {}
                    Err(e) => {
                        self.handle_event(EditorEvent::HandleError(format!(
                            "Failed to finish tar archive: {}",
                            e
                        )));
                        return None;
                    }
                }

                let data = match archive.into_inner() {
                    Ok(data) => data,
                    Err(e) => {
                        self.handle_event(EditorEvent::HandleError(format!(
                            "Failed to get tar archive data: {}",
                            e
                        )));
                        return None;
                    }
                };

                let dispatcher = self.ctx.dispatcher.clone();
                let save = self
                    .ctx
                    .platform
                    .save_file("tilemap.tar", &["tar"], data.as_slice());
                self.ctx.platform.execute_async(async move {
                    match save.await {
                        Ok(_) => {}
                        Err(e) => {
                            dispatcher.send_event(StateLogicDeferredEvent::Editor(
                                EditorEvent::HandleError(format!(
                                    "Failed to save tar archive: {}",
                                    e
                                )),
                            ));
                        }
                    }
                });
            }
        }

        None
    }
}
