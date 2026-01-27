use crate::logic::game_state::LoadTileMenuStateErrors::{
    TilesHeighIsZero, TilesHighMustDivideTexture, TilesWideIsZero, TilesWideMustDivideTexture,
    TilesetNameAlreadyInUse, TilesetNameIsEmpty,
};
use crate::logic::game_state::{
    ApplicationStateEx, Input, LoadedTileSet, TextFieldState, TryLoadTileSetResult, btn, input,
    label, labelled_input, suppress_enter_key,
};
use crate::logic::{ApplicationContext, ApplicationEvent, Grid, GridParams};
use crate::{UI_FONT, UI_FONT_SIZE};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use thiserror::Error;
use vn_scene::TextureId;
use vn_ui::*;

#[derive(Debug)]
pub struct LoadedTexture {
    pub suggested_name: String,
    pub id: TextureId,
    pub dimensions: (u32, u32),
}

#[derive(Debug)]
pub struct LoadTileSetMenuState {
    already_loaded_tilesets: Vec<String>,
    tileset_name_input_state: TextFieldState,
    loaded_texture: LoadedTexture,
    loaded_texture_scroll_x: ScrollBarParams,
    loaded_texture_scroll_y: ScrollBarParams,
    tiles_wide_input: TextFieldState,
    tiles_wide: u32,
    tiles_heigh_input: TextFieldState,
    tiles_high: u32,
    errors: HashSet<LoadTileMenuStateErrors>,
}

#[derive(Debug, Error, Hash, PartialEq, Eq)]
pub enum LoadTileMenuStateErrors {
    #[error("Tiles high must not be 0 or empty")]
    TilesHeighIsZero,
    #[error("Tiles heigh must divide textures width")]
    TilesHeighMustDivideTexture,
    #[error("Tiles wide must not be 0 or empty")]
    TilesWideIsZero,
    #[error("Tiles wide must divide textures width")]
    TilesWideMustDivideTexture,
    #[error("Tiles high must divide textures height")]
    TilesHighMustDivideTexture,
    #[error("Tileset name must not be empty")]
    TilesetNameIsEmpty,
    #[error("Tileset name must be unique")]
    TilesetNameAlreadyInUse,
}

#[derive(Clone, Debug)]
pub enum LoadTileSetMenuInputEvent {
    CaretMoved(usize),
    TextChanged(String),
}

#[derive(Clone, Debug)]
pub enum LoadTileSetMenuEvent {
    Reuse(String),
    Save,
    Cancel,
    TileSetNameInputChanged(LoadTileSetMenuInputEvent),
    TileWideInputChanged(LoadTileSetMenuInputEvent),
    TilesWideChanged(u32),
    TileHeighInputChanged(LoadTileSetMenuInputEvent),
    TilesHighChanged(u32),
    TexturePreviewScrollX(f32),
    TexturePreviewScrollY(f32),
}

pub struct LoadTileSetMenu {
    #[allow(unused)]
    ctx: ApplicationContext,
    ui: RefCell<Box<dyn Element<State = LoadTileSetMenuState, Message = LoadTileSetMenuEvent>>>,
    state: LoadTileSetMenuState,
    event_manager: Rc<RefCell<EventManager>>,
}

impl LoadTileSetMenu {
    pub async fn new(
        ctx: ApplicationContext,
        loaded_texture: LoadedTexture,
        already_loaded_tilesets: Vec<String>,
    ) -> anyhow::Result<Self> {
        let world = Rc::new(RefCell::new(ElementWorld::new()));
        let save = btn(
            "Save",
            UI_FONT,
            UI_FONT_SIZE,
            |state: &LoadTileSetMenuState| !state.errors.is_empty(),
            |_| Color::WHITE,
            ctx.text_metrics.clone(),
            EventHandler::new(|_, _| vec![LoadTileSetMenuEvent::Save]),
            world.clone(),
        );
        let cancel = btn(
            "Cancel",
            UI_FONT,
            UI_FONT_SIZE,
            |_| false,
            |_| Color::WHITE,
            ctx.text_metrics.clone(),
            EventHandler::new(|_, _| vec![LoadTileSetMenuEvent::Cancel]),
            world.clone(),
        );

        let actions = Flex::new(
            {
                let children = vec![
                    FlexChild::new(save).into_rc_refcell(),
                    FlexChild::new(
                        Empty::new(world.clone())
                            .padding(params!(PaddingParams::horizontal(16.0)), world.clone()),
                    )
                    .into_rc_refcell(),
                    FlexChild::new(cancel).into_rc_refcell(),
                ];
                params!(FlexParams {
                    direction: FlexDirection::Row,
                    force_orthogonal_same_size: true,
                    children: children.clone(),
                })
            },
            world.clone(),
        )
        .anchor(
            params!(AnchorParams {
                location: AnchorLocation::Right
            }),
            world.clone(),
        );

        let Input {
            id: tileset_name_input_id,
            element: tileset_name_input,
        } = input(
            |state: &LoadTileSetMenuState| state.tileset_name_input_state.clone(),
            Some("Tileset Name"),
            UI_FONT,
            UI_FONT_SIZE,
            ctx.text_metrics.clone(),
            EventHandler::new(|_, event| match event {
                TextFieldAction::TextChange(new_text) => {
                    vec![LoadTileSetMenuEvent::TileSetNameInputChanged(
                        LoadTileSetMenuInputEvent::TextChanged(new_text),
                    )]
                }
                TextFieldAction::CaretMove(position) => {
                    vec![LoadTileSetMenuEvent::TileSetNameInputChanged(
                        LoadTileSetMenuInputEvent::CaretMoved(position),
                    )]
                }
            })
            .with_overwrite(suppress_enter_key()),
            world.clone(),
        );

        // these could be dropboxes containing all divisors of the texture dimension instead
        let Input {
            id: tiles_wide_id,
            element: tiles_wide,
        } = labelled_input(
            |state: &LoadTileSetMenuState| state.tiles_wide_input.clone(),
            "Tiles Wide: ",
            UI_FONT,
            UI_FONT_SIZE,
            ctx.text_metrics.clone(),
            EventHandler::new(|_, event| match event {
                TextFieldAction::TextChange(new_text) => {
                    vec![LoadTileSetMenuEvent::TileWideInputChanged(
                        LoadTileSetMenuInputEvent::TextChanged(new_text),
                    )]
                }
                TextFieldAction::CaretMove(position) => {
                    vec![LoadTileSetMenuEvent::TileWideInputChanged(
                        LoadTileSetMenuInputEvent::CaretMoved(position),
                    )]
                }
            })
            .with_overwrite(suppress_enter_key()),
            world.clone(),
        );

        let Input {
            id: tiles_heigh_id,
            element: tiles_high,
        } = labelled_input(
            |state: &LoadTileSetMenuState| state.tiles_heigh_input.clone(),
            "Tiles High: ",
            UI_FONT,
            UI_FONT_SIZE,
            ctx.text_metrics.clone(),
            EventHandler::new(|_, event| match event {
                TextFieldAction::TextChange(new_text) => {
                    vec![LoadTileSetMenuEvent::TileHeighInputChanged(
                        LoadTileSetMenuInputEvent::TextChanged(new_text),
                    )]
                }
                TextFieldAction::CaretMove(position) => {
                    vec![LoadTileSetMenuEvent::TileHeighInputChanged(
                        LoadTileSetMenuInputEvent::CaretMoved(position),
                    )]
                }
            })
            .with_overwrite(suppress_enter_key()),
            world.clone(),
        );

        let error = label(
            |state: &LoadTileSetMenuState| {
                let mut messages: Vec<_> = state.errors.iter().map(|e| e.to_string()).collect();
                messages.sort();
                messages.join("\n")
            },
            UI_FONT,
            UI_FONT_SIZE,
            Color::RED,
            ctx.text_metrics.clone(),
            world.clone(),
        );

        let tex_description = label(
            |state: &LoadTileSetMenuState| {
                format!("Dimension:\n {:?}", state.loaded_texture.dimensions)
            },
            UI_FONT,
            UI_FONT_SIZE,
            Color::WHITE.with_alpha(0.5),
            ctx.text_metrics.clone(),
            world.clone(),
        );

        let grid = Grid::new(
            params!(args<LoadTileSetMenuState> => GridParams {
                cols: args.state.tiles_wide,
                rows: args.state.tiles_high,
                grid_size: (args.state.loaded_texture.dimensions.0 as f32 / args.state.tiles_wide as f32, args.state.loaded_texture.dimensions.1 as f32 / args.state.tiles_high as f32),
                grid_color: Color::WHITE,
                grid_width: 3.0,
            }),
            world.clone(),
        );

        // make this scrollable
        // put text with meta information below (specifically the dimensions)
        let texture = PreferSize::new(
            Box::new(ScrollArea::new(
                Box::new(Stack::new(
                    vec![
                        Box::new(Texture::new(
                            params!(args<LoadTileSetMenuState> =>
                                TextureParams {
                                    texture_id: args.state.loaded_texture.id.clone(),
                                    tint: Color::WHITE,
                                    fit_strategy: FitStrategy::Clip {rotation: 0.0},
                                    uv_rect: Rect {
                                        position: [0.0, 0.0],
                                        size: [1.0, 1.0],
                                    },
                                    preferred_size: ElementSize {
                                        width: args.state.loaded_texture.dimensions.0 as f32,
                                        height: args.state.loaded_texture.dimensions.1 as f32,
                                    }
                            }),
                            world.clone(),
                        )),
                        Box::new(grid),
                    ],
                    world.clone(),
                )),
                params!(args<LoadTileSetMenuState> =>
                    ScrollAreaParams {
                        scroll_x: args.state.loaded_texture_scroll_x.clone(),
                        scroll_y: args.state.loaded_texture_scroll_y.clone(),
                        scroll_action_handler: EventHandler::new(|_, e| {
                                match e {
                                    ScrollAreaAction::ScrollX(v) => vec![LoadTileSetMenuEvent::TexturePreviewScrollX(v)],
                                    ScrollAreaAction::ScrollY(v) => vec![LoadTileSetMenuEvent::TexturePreviewScrollY(v)],
                                }
                            }),
                    }
                ),
                world.clone(),
            )),
            params!(PreferSizeParams {
                width: Some(256.0 + 24.0),
                height: Some(256.0 + 24.0),
            }),
            world.clone(),
        );

        let title = Padding::new(
            label(
                |_| "Configure Tileset".to_string(),
                UI_FONT,
                UI_FONT_SIZE,
                Color::WHITE,
                ctx.text_metrics.clone(),
                world.clone(),
            ),
            params!(PaddingParams {
                pad_bottom: 25.0,
                ..Default::default()
            }),
            world.clone(),
        )
        .anchor(
            params!(AnchorParams {
                location: AnchorLocation::Top
            }),
            world.clone(),
        );

        let settings_children = vec![
            FlexChild::new(tileset_name_input).into_rc_refcell(),
            FlexChild::new(tiles_wide).into_rc_refcell(),
            FlexChild::new(tiles_high).into_rc_refcell(),
        ];

        let preview_children = vec![
            FlexChild::new(texture).into_rc_refcell(),
            FlexChild::new(tex_description).into_rc_refcell(),
        ];

        let main_panel_children = vec![
            FlexChild::weighted(
                Flex::new(
                    {
                        params!(FlexParams {
                            direction: FlexDirection::Column,
                            force_orthogonal_same_size: true,
                            children: settings_children.clone(),
                        })
                    },
                    world.clone(),
                ),
                1.0,
            )
            .into_rc_refcell(),
            FlexChild::weighted(
                Flex::new(
                    {
                        params!(FlexParams {
                            direction: FlexDirection::Column,
                            force_orthogonal_same_size: true,
                            children: preview_children.clone()
                        })
                    },
                    world.clone(),
                ),
                1.0,
            )
            .into_rc_refcell(),
        ];

        let main_layout_children = vec![
            FlexChild::new(title).into_rc_refcell(),
            FlexChild::weighted(
                Flex::new(
                    {
                        params!(FlexParams {
                            direction: FlexDirection::Row,
                            force_orthogonal_same_size: true,
                            children: main_panel_children.clone(),
                        })
                    },
                    world.clone(),
                ),
                1.0,
            )
            .into_rc_refcell(),
            FlexChild::weighted(Empty::new(world.clone()), 0.0).into_rc_refcell(),
            FlexChild::new(error).into_rc_refcell(),
            FlexChild::new(actions).into_rc_refcell(),
        ];

        let ui = PreferSize::new(
            Flex::new(
                {
                    params!(FlexParams {
                        direction: FlexDirection::Column,
                        force_orthogonal_same_size: true,
                        children: main_layout_children.clone(),
                    })
                },
                world.clone(),
            ),
            params!(PreferSizeParams {
                width: Some(256.0 * 2.0 + 50.0),
                height: Some(256.0 * 2.0),
            }),
            world.clone(),
        )
        .padding(params!(PaddingParams::uniform(25.0)), world.clone())
        .card(
            params!(CardParams {
                background_color: Color::BLACK,
                border_size: 2.0,
                corner_radius: 5.0,
                border_color: Color::WHITE,
            }),
            world.clone(),
        )
        .anchor(
            params!(AnchorParams {
                location: AnchorLocation::Center
            }),
            world.clone(),
        );

        let mut suggested_name = loaded_texture
            .suggested_name
            .split("[\\\\/]")
            .collect::<Vec<_>>()
            .last()
            .unwrap()
            .to_string();
        if suggested_name.contains("\\.") {
            suggested_name = suggested_name
                .split("\\.")
                .collect::<Vec<_>>()
                .first()
                .unwrap()
                .to_string();
        }

        let mut errors = HashSet::new();
        if suggested_name.is_empty() {
            errors.insert(TilesetNameIsEmpty);
        }
        if already_loaded_tilesets.contains(&suggested_name) {
            errors.insert(TilesetNameAlreadyInUse);
        }
        errors.insert(TilesHeighIsZero);
        errors.insert(TilesWideIsZero);

        Ok(Self {
            ctx,
            ui: RefCell::new(Box::new(ui)),
            state: LoadTileSetMenuState {
                already_loaded_tilesets,
                tileset_name_input_state: TextFieldState {
                    id: tileset_name_input_id,
                    text: suggested_name,
                    caret: None,
                },
                loaded_texture,
                tiles_high: 1,
                tiles_heigh_input: TextFieldState {
                    id: tiles_heigh_id,
                    text: "".to_string(),
                    caret: None,
                },
                tiles_wide: 1,
                tiles_wide_input: TextFieldState {
                    id: tiles_wide_id,
                    text: "".to_string(),
                    caret: None,
                },
                loaded_texture_scroll_x: ScrollBarParams {
                    position: Some(0.0),
                    width: 16.0,
                    margin: 8.0,
                    color: Color::WHITE,
                },
                loaded_texture_scroll_y: ScrollBarParams {
                    position: Some(0.0),
                    width: 16.0,
                    margin: 8.0,
                    color: Color::WHITE,
                },
                errors,
            },
            event_manager: Rc::new(RefCell::new(EventManager::new())),
        })
    }
}

impl ApplicationStateEx for LoadTileSetMenu {
    type StateEvent = LoadTileSetMenuEvent;
    type State = LoadTileSetMenuState;
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
        log::info!("handling event: {:?}", event);
        match event {
            LoadTileSetMenuEvent::TileSetNameInputChanged(event) => match event {
                LoadTileSetMenuInputEvent::CaretMoved(mut position) => {
                    if self.state.tileset_name_input_state.text.is_empty() {
                        position = 0
                    }
                    self.state.tileset_name_input_state.caret = Some(position);
                }
                LoadTileSetMenuInputEvent::TextChanged(new_text) => {
                    if new_text.is_empty() {
                        self.state.errors.insert(TilesetNameIsEmpty);
                    } else {
                        self.state.errors.remove(&TilesetNameIsEmpty);
                    }

                    if self.state.already_loaded_tilesets.contains(&new_text) {
                        self.state.errors.insert(TilesetNameAlreadyInUse);
                    } else {
                        self.state.errors.remove(&TilesetNameAlreadyInUse);
                    }

                    self.state.tileset_name_input_state.text = new_text;
                }
            },
            LoadTileSetMenuEvent::TileWideInputChanged(event) => match event {
                LoadTileSetMenuInputEvent::CaretMoved(mut position) => {
                    if self.state.tiles_wide_input.text.is_empty() {
                        position = 0
                    }
                    self.state.tiles_wide_input.caret =
                        Some(position.min(self.state.tiles_wide_input.text.chars().count()));
                }
                LoadTileSetMenuInputEvent::TextChanged(new_text) => {
                    let new_text = new_text.trim().to_string();
                    if new_text.is_empty() {
                        self.state.tiles_wide = 0;
                        self.state.tiles_wide_input.text = new_text;
                        self.handle_event(LoadTileSetMenuEvent::TilesWideChanged(0));
                    } else {
                        let wide = new_text.parse::<u32>();
                        match wide {
                            Ok(wide) => {
                                self.state.tiles_wide = wide;
                                self.state.tiles_wide_input.text = new_text;
                                self.handle_event(LoadTileSetMenuEvent::TilesWideChanged(wide));
                            }
                            Err(_) => {}
                        }
                    }
                }
            },
            LoadTileSetMenuEvent::TileHeighInputChanged(event) => match event {
                LoadTileSetMenuInputEvent::CaretMoved(mut position) => {
                    if self.state.tiles_heigh_input.text.is_empty() {
                        position = 0
                    }
                    self.state.tiles_heigh_input.caret =
                        Some(position.min(self.state.tiles_heigh_input.text.chars().count()));
                }
                LoadTileSetMenuInputEvent::TextChanged(new_text) => {
                    let new_text = new_text.trim().to_string();
                    if new_text.is_empty() {
                        self.state.tiles_high = 0;
                        self.state.tiles_heigh_input.text = new_text;
                        self.handle_event(LoadTileSetMenuEvent::TilesHighChanged(0));
                    } else {
                        let heigh = new_text.parse::<u32>();
                        match heigh {
                            Ok(heigh) => {
                                self.state.tiles_high = heigh;
                                self.state.tiles_heigh_input.text = new_text;
                                self.handle_event(LoadTileSetMenuEvent::TilesHighChanged(heigh));
                            }
                            Err(_) => {}
                        }
                    }
                }
            },
            LoadTileSetMenuEvent::TilesWideChanged(wide) => {
                if wide == 0 {
                    self.state.errors.insert(TilesWideIsZero);
                } else {
                    self.state.errors.remove(&TilesWideIsZero);
                    if self.state.loaded_texture.dimensions.0.is_multiple_of(wide) {
                        self.state.errors.remove(&TilesWideMustDivideTexture);
                    } else {
                        self.state.errors.insert(TilesWideMustDivideTexture);
                    }
                }
            }
            LoadTileSetMenuEvent::TilesHighChanged(high) => {
                if high == 0 {
                    self.state.errors.insert(TilesHeighIsZero);
                } else {
                    self.state.errors.remove(&TilesHeighIsZero);
                    if self.state.loaded_texture.dimensions.1.is_multiple_of(high) {
                        self.state.errors.remove(&TilesHighMustDivideTexture);
                    } else {
                        self.state.errors.insert(TilesHighMustDivideTexture);
                    }
                }
            }
            LoadTileSetMenuEvent::TexturePreviewScrollX(v) => {
                self.state.loaded_texture_scroll_x.position = Some(v);
            }
            LoadTileSetMenuEvent::TexturePreviewScrollY(v) => {
                self.state.loaded_texture_scroll_y.position = Some(v);
            }

            LoadTileSetMenuEvent::Save => {
                return Some(ApplicationEvent::TilesetLoaded(
                    TryLoadTileSetResult::Loaded(LoadedTileSet {
                        texture_id: self.state.loaded_texture.id.clone(),
                        name: self.state.tileset_name_input_state.text.clone(),
                        texture_dimensions: self.state.loaded_texture.dimensions,
                        tile_dimensions: (
                            self.state.loaded_texture.dimensions.0 / self.state.tiles_wide,
                            self.state.loaded_texture.dimensions.1 / self.state.tiles_high,
                        ),
                    }),
                ));
            }
            LoadTileSetMenuEvent::Reuse(name) => return Some(ApplicationEvent::TilesetReuse(name)),
            LoadTileSetMenuEvent::Cancel => return Some(ApplicationEvent::TilesetLoadCanceled),
        }

        None
    }
}
