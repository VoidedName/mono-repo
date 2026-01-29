use crate::logic::app_state::editor_ui::{editor, layers, tileset};
use crate::logic::app_state::{
    ApplicationStateEx, LoadedTileSet, TryLoadTileSetResult, label, with_fps,
};
use crate::logic::{ApplicationContext, ApplicationEvent, EditorCallback};
use crate::{UI_FONT, UI_FONT_SIZE};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use vn_scene::Color;
use vn_tilemap::{TileMapLayerMapSpecification, TileMapLayerSpecification, TileMapSpecification};
use vn_ui::{
    AnchorExt, AnchorLocation, AnchorParams, Element, ElementWorld, Empty, EventManager, Flex,
    FlexChild, FlexDirection, FlexParams, PaddingExt, PaddingParams, ScrollBarParams, params,
};

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
}

#[derive(Debug, Clone)]
pub enum EditorEvent {
    TilesetViewScrollX(f32),
    TilesetViewScrollY(f32),
    TilemapViewScrollX(f32),
    TilemapViewScrollY(f32),
    TryAddingLayer,
    LoadSpec,
    SaveSpec,
    AddLayer(TryLoadTileSetResult),
    SwitchToLayer(usize),
    DeleteLayer(usize),
    LayerCaretPosition(usize, usize),
    RenameLayer(usize, String),
    TileBrushSelect(Brush),
    Brushing(u32, u32),
    ChangeTilemapSize(u32, u32),
}

#[derive(Debug, Clone)]
pub enum Brush {
    Tileset((u32, u32), (u32, u32)),
    Eraser,
    None,
}

pub struct Editor {
    #[allow(unused)]
    ctx: ApplicationContext,
    ui: RefCell<Box<dyn Element<State = EditorState, Message = EditorEvent>>>,
    state: EditorState,
    event_manager: Rc<RefCell<EventManager>>,
}

impl Editor {
    pub async fn new(ctx: ApplicationContext) -> anyhow::Result<Self> {
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

        let ui = Flex::new(
            {
                let children = vec![
                    FlexChild::new(title).into_rc_refcell(),
                    FlexChild::weighted(
                        Flex::new(
                            {
                                let children = vec![
                                    FlexChild::new(layers).into_rc_refcell(),
                                    FlexChild::new(Empty::new(world.clone()).padding(
                                        params!(PaddingParams::horizontal(25.0)),
                                        world.clone(),
                                    ))
                                    .into_rc_refcell(),
                                    FlexChild::weighted(editor, 1.0).into_rc_refcell(),
                                    FlexChild::new(Empty::new(world.clone()).padding(
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

        let scroll_bar = ScrollBarParams {
            width: 16.0,
            color: Color::WHITE,
            position: Some(0.0),
            margin: 8.0,
        };

        Ok(Self {
            ui: RefCell::new(with_fps(&ctx, Box::new(ui), world.clone())),
            ctx,
            state: EditorState {
                brush: Brush::None,
                layer_caret_positions: Vec::new(),
                current_layer: None,
                loaded_tilesets: HashMap::new(),
                tile_map: TileMapSpecification {
                    layers: vec![],
                    map_dimensions: (10, 5),
                },
                tileset_view_scroll_x: scroll_bar,
                tileset_view_scroll_y: scroll_bar,
                tilemap_view_scroll_y: scroll_bar,
                tilemap_view_scroll_x: scroll_bar,
            },
            event_manager: Rc::new(RefCell::new(EventManager::new())),
        })
    }
}

impl ApplicationStateEx for Editor {
    type StateEvent = EditorEvent;
    type State = EditorState;
    type ApplicationEvent = ApplicationEvent;

    fn ui(&self) -> &RefCell<Box<dyn Element<State = Self::State, Message = Self::StateEvent>>> {
        &self.ui
    }

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn event_manager(&self) -> Rc<RefCell<EventManager>> {
        self.event_manager.clone()
    }

    fn handle_event(&mut self, event: Self::StateEvent) -> Option<Self::ApplicationEvent> {
        log::info!("handling state event: {:?}", event);

        match event {
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
            EditorEvent::LoadSpec => {}
            EditorEvent::SaveSpec => {}
        }

        None
    }
}
