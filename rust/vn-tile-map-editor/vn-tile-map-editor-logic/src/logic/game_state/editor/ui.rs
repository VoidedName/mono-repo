use crate::logic::game_state::editor::grid::GridParams;
use crate::logic::game_state::editor::{Editor, EditorEvent, Grid};
use std::rc::Rc;
use vn_scene::{Color, Rect};
use vn_ui::{
    Anchor, AnchorExt, AnchorLocation, AnchorParams, ButtonExt, ButtonParams, Card, CardExt,
    CardParams, Element, ElementId, ElementSize, ElementWorld, EventHandler, FitStrategy, Flex,
    FlexChild, InteractionEventKind, InteractionState, InteractiveExt, PaddingExt, PaddingParams,
    ScrollAreaExt, ScrollAreaParams, ScrollBarParams, Stack, StateToParamsArgs, TextField,
    TextFieldAction, TextFieldParams, TextMetrics, TextVisuals, Texture, TextureParams, params,
};
use vn_wgpu_window::resource_manager::ResourceManager;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard;
use winit::keyboard::NamedKey;

pub const UI_FONT: &str = "jetbrains-bold";
pub const UI_FONT_SIZE: f32 = 16.0;

pub struct EditorUi<ApplicationEvent> {
    pub root: Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>>,
    pub tileset_path_input_id: ElementId,
    pub tile_width_input_id: ElementId,
    pub tile_height_input_id: ElementId,
    pub tileset_cols_input_id: ElementId,
    pub tileset_rows_input_id: ElementId,
    pub tileset_preview_scroll_area_id: ElementId,
}

pub fn build_editor_ui<ApplicationEvent: 'static>(
    editor: &Editor<ApplicationEvent>,
    world: &mut ElementWorld,
    rm: Rc<ResourceManager>,
    metrics: Rc<dyn TextMetrics>,
) -> EditorUi<ApplicationEvent> {
    let title = build_title(world, metrics.clone());
    let grid = build_grid(world);
    let sidebar_info = build_sidebar(editor, world, metrics.clone());
    let (preview, tileset_preview_scroll_area_id) = build_tileset_preview_panel(editor, world);
    let fps_counter = build_fps_counter(metrics.clone(), world);

    let rm_ = rm.clone();
    let text_atlas = Texture::new(
        params! { TextureParams {
            texture_id: rm_
                .texture_atlas
                .borrow()
                .atlases
                .last()
                .unwrap()
                .texture
                .id
                .clone(),
            preferred_size: ElementSize {
                height: 256.0 * 8.0,
                width: 256.0 * 8.0,
            },
            uv_rect: Rect {
                position: [0.0, 0.0],
                size: [1.0, 1.0],
            },
            tint: Color::WHITE.with_alpha(0.5),
            fit_strategy: FitStrategy::Clip { rotation: 0.0 },
        }},
        world,
    );

    let main_layout = Flex::new_row(
        vec![
            FlexChild::new(sidebar_info.sidebar),
            FlexChild::weighted(
                Box::new(Anchor::new(
                    grid,
                    params! { AnchorParams {
                        location: AnchorLocation::CENTER,
                    }},
                    world,
                )),
                1.0,
            ),
            FlexChild::new(preview),
        ],
        false,
        world,
    );
    // .padding(move |_| PaddingParams::uniform(40.0), world);

    let ui = Flex::new_column(
        vec![
            FlexChild::new(title),
            FlexChild::weighted(Box::new(main_layout), 1.0),
        ],
        false,
        world,
    );

    let ui = Stack::new(
        vec![
            Box::new(ui),
            Box::new(Anchor::new(
                fps_counter,
                params! { AnchorParams {
                    location: AnchorLocation::TopRight,
                }},
                world,
            )),
            Box::new(text_atlas),
        ],
        world,
    );

    EditorUi {
        root: Box::new(ui),
        tileset_path_input_id: sidebar_info.tileset_path_input_id,
        tile_width_input_id: sidebar_info.tile_width_input_id,
        tile_height_input_id: sidebar_info.tile_height_input_id,
        tileset_cols_input_id: sidebar_info.tileset_cols_input_id,
        tileset_rows_input_id: sidebar_info.tileset_rows_input_id,
        tileset_preview_scroll_area_id,
    }
}

fn build_title<ApplicationEvent: 'static>(
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>> {
    Box::new(
        TextField::new(
            params! { {
                let metrics = metrics.clone();
                 TextFieldParams {
                    visuals: TextVisuals {
                        text: "Tile Map Editor".to_string(),
                        caret_position: None,
                        font: UI_FONT.to_string(),
                        font_size: UI_FONT_SIZE,
                        color: Color::WHITE,
                        caret_width: None,
                        caret_blink_duration: None,
                    },
                    metrics: metrics.clone(),
                    interaction: Default::default(),
                    text_field_action_handler: EventHandler::none(),
                }
            }},
            world,
        )
        .padding(params! { PaddingParams::uniform(10.0) }, world)
        .anchor(
            params! { AnchorParams {
                location: AnchorLocation::TOP,
            } },
            world,
        ),
    )
}

fn build_grid<ApplicationEvent: 'static>(world: &mut ElementWorld) -> Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>> {
    Box::new(
        Grid::new(
            params! { GridParams {
                grid_size: (32.0, 32.0),
                cols: 10,
                rows: 10,
                grid_width: 3.0,
                grid_color: Color::WHITE.with_alpha(0.5),
            }},
            world,
        )
        .padding(params! { PaddingParams::uniform(10.0) }, world)
        .card(
            params! { CardParams {
                background_color: Color::BLACK.with_alpha(0.3),
                border_size: 2.0,
                border_color: Color::WHITE.with_alpha(0.5),
                corner_radius: 5.0,
            }},
            world,
        ),
    )
}

pub struct SidebarInfo<ApplicationEvent> {
    pub sidebar: Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>>,
    pub tileset_path_input_id: ElementId,
    pub tile_width_input_id: ElementId,
    pub tile_height_input_id: ElementId,
    pub tileset_cols_input_id: ElementId,
    pub tileset_rows_input_id: ElementId,
}

pub struct TilesetViewInfo<ApplicationEvent> {
    pub element: Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>>,
    pub tileset_path_input_id: ElementId,
    pub tile_width_input_id: ElementId,
    pub tile_height_input_id: ElementId,
    pub tileset_cols_input_id: ElementId,
    pub tileset_rows_input_id: ElementId,
}

fn build_sidebar<ApplicationEvent: 'static>(
    editor: &Editor<ApplicationEvent>,
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> SidebarInfo<ApplicationEvent> {
    let metrics_ = metrics.clone();
    let sidebar_title = Box::new(TextField::new(
        params! { {
             TextFieldParams {
                visuals: TextVisuals {
                    text: "Layers".to_string(),
                    caret_position: None,
                    font: UI_FONT.to_string(),
                    font_size: UI_FONT_SIZE,
                    color: Color::WHITE,
                    caret_width: None,
                    caret_blink_duration: None,
                },
                metrics: metrics_.clone(),
                interaction: Default::default(),
                text_field_action_handler: EventHandler::none(),
            }
        }},
        world,
    ));

    let layer_list = build_layer_list(editor, world, metrics.clone());
    let add_layer_button = build_add_layer_button(editor, world, metrics.clone());
    let tileset_title = build_tileset_title(world, metrics.clone());
    let tileset_view_info = build_tileset_view(editor, world, metrics.clone());
    let selection_info = build_selection_info(editor, world, metrics.clone());
    let footer = build_footer(editor, world, metrics.clone());

    let sidebar = Box::new(
        Flex::new_column_unweighted(
            vec![
                sidebar_title,
                layer_list,
                add_layer_button,
                tileset_title,
                tileset_view_info.element,
                selection_info,
                footer,
            ],
            true,
            world,
        )
        .padding(params! { PaddingParams::uniform(10.0) }, world)
        .card(
            params! {CardParams {
                background_color: Color::BLACK.with_alpha(0.5),
                border_size: 2.0,
                border_color: Color::WHITE.with_alpha(0.5),
                corner_radius: 5.0,
            }},
            world,
        ),
    );

    SidebarInfo {
        sidebar,
        tileset_path_input_id: tileset_view_info.tileset_path_input_id,
        tile_width_input_id: tileset_view_info.tile_width_input_id,
        tile_height_input_id: tileset_view_info.tile_height_input_id,
        tileset_cols_input_id: tileset_view_info.tileset_cols_input_id,
        tileset_rows_input_id: tileset_view_info.tileset_rows_input_id,
    }
}

fn build_layer_list<ApplicationEvent: 'static>(
    editor: &Editor<ApplicationEvent>,
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>> {
    let mut layer_elements: Vec<Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>>> =
        Vec::new();
    for (i, _layer) in editor.map_spec.layers.iter().enumerate() {
        let is_selected = i == editor.selected_layer_index;

        let layer_label = TextField::new(
            {
                let metrics = metrics.clone();
                params! {
                     TextFieldParams {
                        visuals: TextVisuals {
                            text: format!("Layer {}", i),
                            caret_position: None,
                            font: UI_FONT.to_string(),
                            font_size: UI_FONT_SIZE,
                            color: if is_selected {
                                Color::RED
                            } else {
                                Color::WHITE
                            },
                            caret_width: None,
                            caret_blink_duration: None,
                        },
                        metrics: metrics.clone(),
                        interaction: Default::default(),
                        text_field_action_handler: EventHandler::none(),
                    }
                }
            },
            world,
        )
        .interactive_set(false, world);

        let remove_button = TextField::new(
            {
                let metrics = metrics.clone();
                params! {
                     TextFieldParams {
                        visuals: TextVisuals {
                            text: "X".to_string(),
                            caret_position: None,
                            font: UI_FONT.to_string(),
                            font_size: UI_FONT_SIZE,
                            color: Color::RED,
                            caret_width: None,
                            caret_blink_duration: None,
                        },
                        metrics: metrics.clone(),
                        interaction: Default::default(),
                        text_field_action_handler: EventHandler::none(),
                    }
                }
            },
            world,
        )
        .interactive_set(false, world)
        .padding(params! {PaddingParams::uniform(2.0) }, world)
        .button(
            move |args: StateToParamsArgs<'_, Editor<ApplicationEvent>>| ButtonParams {
                background: Color::BLACK.with_alpha(0.3),
                border_color: if args.ctx.event_manager.borrow().is_hovered(args.id) {
                    Color::RED
                } else {
                    Color::TRANSPARENT
                },
                border_width: 2.0,
                corner_radius: 2.0,
                interaction: InteractionState {
                    is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                    is_focused: false,
                },
                on_click: EditorEvent::RemoveLayer(i).into(),
            },
            world,
        );

        let layer_row = Flex::new_row_unweighted(
            vec![Box::new(layer_label), Box::new(remove_button)],
            false,
            world,
        )
        .padding(params! {PaddingParams::uniform(5.0) }, world)
        .button(
            {
                let i = i;
                move |args: StateToParamsArgs<'_, Editor<ApplicationEvent>>| ButtonParams {
                    background: if is_selected {
                        Color::WHITE.with_alpha(0.2)
                    } else {
                        Color::WHITE.with_alpha(0.1)
                    },
                    border_color: if args.ctx.event_manager.borrow().is_hovered(args.id) {
                        Color::WHITE
                    } else {
                        Color::WHITE.with_alpha(0.3)
                    },
                    border_width: 2.0,
                    corner_radius: 3.0,
                    interaction: InteractionState {
                        is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                        is_focused: false,
                    },
                    on_click: Some(EditorEvent::SelectLayer(i)).into(),
                }
            },
            world,
        );

        layer_elements.push(Box::new(layer_row));
    }

    Box::new(Flex::new_column_unweighted(layer_elements, false, world))
}

fn build_add_layer_button<ApplicationEvent: 'static>(
    _editor: &Editor<ApplicationEvent>,
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>> {
    let button = TextField::new(
        params! {{
            let metrics = metrics.clone();
             TextFieldParams {
                visuals: TextVisuals {
                    text: "Add Layer".to_string(),
                    caret_position: None,
                    font: UI_FONT.to_string(),
                    font_size: UI_FONT_SIZE,
                    color: Color::WHITE,
                    caret_width: None,
                    caret_blink_duration: None,
                },
                metrics: metrics.clone(),
                interaction: Default::default(),
                text_field_action_handler: EventHandler::none(),
            }
        }},
        world,
    )
    .interactive_set(false, world)
    .padding(params! {PaddingParams::uniform(5.0)}, world)
    .button(
        params! {args => ButtonParams {
            background: Color::WHITE.with_alpha(0.1),
            border_color: if args.ctx.event_manager.borrow().is_hovered(args.id) {
                Color::WHITE
            } else {
                Color::WHITE.with_alpha(0.3)
            },
            border_width: 2.0,
            corner_radius: 3.0,
            interaction: InteractionState {
                is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                is_focused: false,
            },
            on_click: Some(EditorEvent::AddLayer).into(),
        }},
        world,
    );

    Box::new(button)
}

fn build_tileset_title<ApplicationEvent: 'static>(
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>> {
    Box::new(TextField::new(
        params! {{
            let metrics = metrics.clone();
             TextFieldParams {
                visuals: TextVisuals {
                    text: "Tileset".to_string(),
                    caret_position: None,
                    font: UI_FONT.to_string(),
                    font_size: UI_FONT_SIZE,
                    color: Color::WHITE,
                    caret_width: None,
                    caret_blink_duration: None,
                },
                metrics: metrics.clone(),
                interaction: Default::default(),
                text_field_action_handler: EventHandler::none(),
            }
        }},
        world,
    ))
}

fn build_dimension_input<ApplicationEvent: 'static>(
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
    label: String,
    text: fn(&Editor<ApplicationEvent>) -> String,
    caret: fn(&Editor<ApplicationEvent>) -> usize,
    on_action: Option<fn(ElementId, TextFieldAction) -> EditorEvent>,
) -> (
    Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>>,
    ElementId,
) {
    let label_el = TextField::new(
        {
            let metrics = metrics.clone();
            params! {
                 TextFieldParams {
                    visuals: TextVisuals {
                        text: label.clone(),
                        caret_position: None,
                        font: UI_FONT.to_string(),
                        font_size: UI_FONT_SIZE,
                        color: Color::WHITE.with_alpha(0.7),
                        caret_width: None,
                        caret_blink_duration: None,
                    },
                    metrics: metrics.clone(),
                    interaction: Default::default(),
                    text_field_action_handler: EventHandler::none(),
                }
            }
        },
        world,
    )
    .padding(params! { PaddingParams::uniform(5.0) }, world);

    let input = TextField::new(
        {
            let metrics = metrics.clone();
            let text = text.clone();
            params! { args =>
                let is_focused = args.ctx.event_manager.borrow().is_focused(args.id);
                TextFieldParams {
                    visuals: TextVisuals {
                        text: text(args.state),
                        caret_position: if is_focused {
                            Some(caret(args.state))
                        } else {
                            None
                        },
                        font: UI_FONT.to_string(),
                        font_size: UI_FONT_SIZE,
                        color: Color::WHITE,
                        caret_width: Some(2.0),
                        caret_blink_duration: Some(0.5),
                    },
                    metrics: metrics.clone(),
                    interaction: InteractionState {
                        is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                        is_focused,
                    },
                    text_field_action_handler: on_action.map_or(EventHandler::none(), |f| {
                        EventHandler::new(move |a, b| vec![f(a, b)]).with_overwrite(|_, b| match b.kind {
                            InteractionEventKind::Keyboard(KeyEvent {
                                state: ElementState::Pressed,
                                logical_key: keyboard::Key::Named(NamedKey::Enter),
                                ..
                            }) => {
                                return (vec![], false);
                            }
                            _ => (vec![], true),
                        })
                    }),
                }
            }
        },
        world,
    );

    let input_id = input.id();

    let input = input
        .padding(params! {PaddingParams::uniform(5.0) }, world)
        .interactive_set(true, world)
        .card(
            params! {CardParams {
                background_color: Color::BLACK.with_alpha(0.5),
                border_size: 2.0,
                border_color: Color::WHITE.with_alpha(0.3),
                corner_radius: 3.0,
            } },
            world,
        );

    (
        Box::new(Flex::new_row_unweighted(
            vec![Box::new(label_el), Box::new(input)],
            false,
            world,
        )),
        input_id,
    )
}

fn build_tileset_view<ApplicationEvent: 'static>(
    editor: &Editor<ApplicationEvent>,
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> TilesetViewInfo<ApplicationEvent> {
    let mut tileset_elements: Vec<Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>>> =
        Vec::new();
    let current_tileset = editor
        .map_spec
        .layers
        .get(editor.selected_layer_index)
        .map(|l| l.tile_set.clone())
        .unwrap_or_else(|| "none".to_string());

    let current_ts_label = TextField::new(
        {
            let metrics = metrics.clone();
            let current_tileset = current_tileset.clone();
            params! {
                 TextFieldParams {
                    visuals: TextVisuals {
                        text: format!("Current: {}", current_tileset),
                        caret_position: None,
                        font: UI_FONT.to_string(),
                        font_size: UI_FONT_SIZE,
                        color: Color::WHITE.with_alpha(0.7),
                        caret_width: None,
                        caret_blink_duration: None,
                    },
                    metrics: metrics.clone(),
                    interaction: Default::default(),
                    text_field_action_handler: EventHandler::none(),
                }
            }
        },
        world,
    )
    .padding(params! {PaddingParams::uniform(5.0)}, world);
    tileset_elements.push(Box::new(current_ts_label));

    let tileset_input: TextField<Editor<ApplicationEvent>, EditorEvent> = TextField::new(
        {
            let metrics = metrics.clone();
            params! { args<Editor<ApplicationEvent>> =>
                let is_focused = args.ctx.event_manager.borrow().is_focused(args.id);

                TextFieldParams {
                    visuals: TextVisuals {
                        text: args.state.tileset_path.clone(),
                        caret_position: if is_focused {
                            Some(args.state.tileset_path_caret)
                        } else {
                            None
                        },
                        font: UI_FONT.to_string(),
                        font_size: UI_FONT_SIZE,
                        color: Color::WHITE,
                        caret_width: Some(2.0),
                        caret_blink_duration: Some(0.5),
                    },
                    metrics: metrics.clone(),
                    interaction: InteractionState {
                        is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                        is_focused,
                    },
                    text_field_action_handler: EventHandler::new(|id, action| {
                        vec![EditorEvent::TextFieldAction { id, action }]
                    }),
                }
            }
        },
        world,
    );

    let path_input_id = tileset_input.id();

    let tileset_input: Card<Editor<ApplicationEvent>, EditorEvent> = tileset_input
        .padding(params!(PaddingParams::uniform(5.0)), world)
        .interactive_set(true, world)
        .card(
            params! { CardParams {
                background_color: Color::BLACK.with_alpha(0.5),
                border_size: 2.0,
                border_color: Color::WHITE.with_alpha(0.3),
                corner_radius: 3.0,
            }},
            world,
        );
    tileset_elements.push(Box::new(tileset_input));

    let metrics_ = metrics.clone();

    let load_button = TextField::new(
        params! {TextFieldParams {
            visuals: TextVisuals {
                text: "Load Tileset".to_string(),
                caret_position: None,
                font: UI_FONT.to_string(),
                font_size: UI_FONT_SIZE,
                color: Color::WHITE,
                caret_width: None,
                caret_blink_duration: None,
            },
            metrics: metrics_.clone(),
            interaction: Default::default(),
            text_field_action_handler: EventHandler::none(),
        }},
        world,
    )
    .interactive_set(false, world)
    .padding(params! {PaddingParams::uniform(5.0)}, world)
    .button(
        params! {args => ButtonParams {
            background: Color::WHITE.with_alpha(0.1),
            border_color: if args.ctx.event_manager.borrow().is_hovered(args.id) {
                Color::WHITE
            } else {
                Color::WHITE.with_alpha(0.3)
            },
            border_width: 2.0,
            corner_radius: 3.0,
            interaction: InteractionState {
                is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                is_focused: false,
            },
            on_click: Some(EditorEvent::LoadTilesetFromInput).into(),
        }},
        world,
    );

    tileset_elements.push(Box::new(load_button));

    let (tw_input, tw_id) = build_dimension_input(
        world,
        metrics.clone(),
        "Tile W:".to_string(),
        |editor| editor.tile_width_text.clone(),
        |editor| editor.tile_width_caret,
        Some(|id, action| EditorEvent::TextFieldAction { id, action }),
    );
    let (th_input, th_id) = build_dimension_input(
        world,
        metrics.clone(),
        "Tile H:".to_string(),
        |editor| editor.tile_height_text.clone(),
        |editor| editor.tile_height_caret,
        Some(|id, action| EditorEvent::TextFieldAction { id, action }),
    );
    tileset_elements.push(Box::new(Flex::new_row_unweighted(
        vec![tw_input, th_input],
        false,
        world,
    )));

    // Tileset Dimensions (in tiles)
    let (tsw_input, tsw_id) = build_dimension_input(
        world,
        metrics.clone(),
        "Set Cols:".to_string(),
        |editor| editor.tileset_cols_text.clone(),
        |editor| editor.tileset_cols_caret,
        Some(|id, action| EditorEvent::TextFieldAction { id, action }),
    );
    let (tsh_input, tsh_id) = build_dimension_input(
        world,
        metrics.clone(),
        "Set Rows:".to_string(),
        |editor| editor.tileset_rows_text.clone(),
        |editor| editor.tileset_rows_caret,
        Some(|id, action| EditorEvent::TextFieldAction { id, action }),
    );
    tileset_elements.push(Box::new(Flex::new_row_unweighted(
        vec![tsw_input, tsh_input],
        false,
        world,
    )));

    let mut recently_loaded_elements: Vec<Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>>> =
        Vec::new();
    for (path, _) in &editor.loaded_tilesets {
        let metrics = metrics.clone();
        let path = path.clone();
        let is_selected = path == current_tileset;
        let ts_button = TextField::new(
            {
                let path = path.clone();
                params! {
                     TextFieldParams {
                        visuals: TextVisuals {
                            text: path.clone(),
                            caret_position: None,
                            font: UI_FONT.to_string(),
                            font_size: UI_FONT_SIZE,
                            color: if is_selected {
                                Color::RED
                            } else {
                                Color::WHITE
                            },
                            caret_width: None,
                            caret_blink_duration: None,
                        },
                        metrics: metrics.clone(),
                        interaction: Default::default(),
                        text_field_action_handler: EventHandler::none(),
                    }
                }
            },
            world,
        )
        .interactive_set(false, world)
        .padding(params! {PaddingParams::uniform(3.0) }, world)
        .button(
            {
                let path = path.clone();
                move |args: StateToParamsArgs<'_, Editor<ApplicationEvent>>| ButtonParams {
                    background: if is_selected {
                        Color::WHITE.with_alpha(0.2)
                    } else {
                        Color::WHITE.with_alpha(0.1)
                    },
                    border_color: if args.ctx.event_manager.borrow().is_hovered(args.id) {
                        Color::WHITE
                    } else {
                        Color::WHITE.with_alpha(0.3)
                    },
                    border_width: 2.0,
                    corner_radius: 3.0,
                    interaction: InteractionState {
                        is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                        is_focused: false,
                    },
                    on_click: Some(EditorEvent::SelectTileset(path.clone())).into(),
                }
            },
            world,
        );

        recently_loaded_elements.push(Box::new(ts_button));
    }

    if !recently_loaded_elements.is_empty() {
        tileset_elements.push(Box::new(
            TextField::new(
                params! {TextFieldParams {
                    visuals: TextVisuals {
                        text: "Recently Loaded:".to_string(),
                        caret_position: None,
                        font: UI_FONT.to_string(),
                        font_size: UI_FONT_SIZE,
                        color: Color::WHITE.with_alpha(0.7),
                        caret_width: None,
                        caret_blink_duration: None,
                    },
                    metrics: metrics.clone(),
                    interaction: Default::default(),
                    text_field_action_handler: EventHandler::none(),
                }},
                world,
            )
            .padding(params! {PaddingParams::uniform(5.0)}, world),
        ));
        tileset_elements.push(Box::new(Flex::new_column_unweighted(
            recently_loaded_elements,
            false,
            world,
        )));
    }

    let element = Box::new(Flex::new_column_unweighted(tileset_elements, false, world));

    TilesetViewInfo {
        element,
        tileset_path_input_id: path_input_id,
        tile_width_input_id: tw_id,
        tile_height_input_id: th_id,
        tileset_cols_input_id: tsw_id,
        tileset_rows_input_id: tsh_id,
    }
}

fn build_selection_info<ApplicationEvent: 'static>(
    _editor: &Editor<ApplicationEvent>,
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>> {
    let metrics = metrics.clone();
    Box::new(TextField::new(
        move |args: StateToParamsArgs<'_, Editor<ApplicationEvent>>| {
            let layer_count = args.state.map_spec.layers.len();
            let selected = args.state.selected_layer_index;
            TextFieldParams {
                visuals: TextVisuals {
                    text: format!(
                        "Selected Layer: {} / {}",
                        (selected + 1).min(layer_count),
                        layer_count
                    ),
                    caret_position: None,
                    font: UI_FONT.to_string(),
                    font_size: UI_FONT_SIZE,
                    color: Color::WHITE.with_alpha(0.7),
                    caret_width: None,
                    caret_blink_duration: None,
                },
                metrics: metrics.clone(),
                interaction: Default::default(),
                text_field_action_handler: EventHandler::none(),
            }
        },
        world,
    ))
}

fn build_footer<ApplicationEvent: 'static>(
    _editor: &Editor<ApplicationEvent>,
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>> {
    let mut footer_elements: Vec<Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>>> =
        Vec::new();
    for btn_text in ["Save", "Load", "Settings"] {
        let event = match btn_text {
            "Save" => EditorEvent::SaveMap,
            "Load" => EditorEvent::LoadMap,
            "Settings" => EditorEvent::OpenSettings,
            _ => unreachable!(),
        };

        let metrics = metrics.clone();
        let button = TextField::new(
            params! {TextFieldParams {
                visuals: TextVisuals {
                    text: btn_text.to_string(),
                    caret_position: None,
                    font: UI_FONT.to_string(),
                    font_size: UI_FONT_SIZE,
                    color: Color::WHITE,
                    caret_width: None,
                    caret_blink_duration: None,
                },
                metrics: metrics.clone(),
                interaction: Default::default(),
                text_field_action_handler: EventHandler::none(),
            }},
            world,
        )
        .padding(params! {PaddingParams::uniform(5.0)}, world)
        .button(
            move |args: StateToParamsArgs<'_, Editor<ApplicationEvent>>| ButtonParams {
                background: Color::WHITE.with_alpha(0.1),
                border_color: if args.ctx.event_manager.borrow().is_hovered(args.id) {
                    Color::WHITE
                } else {
                    Color::WHITE.with_alpha(0.3)
                },
                border_width: 2.0,
                corner_radius: 3.0,
                interaction: InteractionState {
                    is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                    is_focused: false,
                },
                on_click: Some(event.clone()).into(),
            },
            world,
        );
        footer_elements.push(Box::new(button));
    }
    Box::new(Flex::new_row_unweighted(footer_elements, false, world))
}

fn build_tileset_preview_panel<ApplicationEvent: 'static>(
    editor: &Editor<ApplicationEvent>,
    world: &mut ElementWorld,
) -> (
    Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>>,
    ElementId,
) {
    let mut tileset_preview_elements: Vec<Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>>> =
        Vec::new();
    let mut scroll_area_id = world.next_id(); // Placeholder if no texture

    if let Some(layer) = editor.map_spec.layers.get(editor.selected_layer_index) {
        if let Some(texture_id) = editor.loaded_tilesets.get(&layer.tile_set) {
            let texture_id = texture_id.clone();
            let texture_preview = Texture::new(
                params! {TextureParams {
                    texture_id: texture_id.clone(),
                    preferred_size: ElementSize {
                        width: 256.0,
                        height: 4256.0,
                    },
                    uv_rect: Rect {
                        position: [0.0, 0.0],
                        size: [1.0, 1.0],
                    },
                    tint: Color::WHITE,
                    fit_strategy: FitStrategy::PreserveAspectRatio { rotation: 0.0 },
                }},
                world,
            );

            let grid_overlay = Grid::new(
                params! {GridParams {
                    rows: 133,
                    cols: 8,
                    grid_size: (32.0, 32.0),
                    grid_width: 3.0,
                    grid_color: Color::WHITE.with_alpha(0.5),
                }},
                world,
            );

            let scroll_area = Stack::new(
                vec![Box::new(texture_preview), Box::new(grid_overlay)],
                world,
            )
            .scroll_area(
                move |args: StateToParamsArgs<'_, Editor<ApplicationEvent>>| ScrollAreaParams {
                    scroll_x: ScrollBarParams {
                        width: 16.0,
                        margin: 8.0,
                        color: Color::WHITE.with_alpha(0.5),
                        position: Some(args.state.tileset_scroll_x),
                    },
                    scroll_y: ScrollBarParams {
                        width: 16.0,
                        margin: 8.0,
                        color: Color::WHITE.with_alpha(0.5),
                        position: Some(args.state.tileset_scroll_y),
                    },
                    scroll_action_handler: EventHandler::new(|id, action| {
                        vec![EditorEvent::ScrollAction { id, action }]
                    }),
                },
                world,
            );

            scroll_area_id = scroll_area.id();

            let preview_card = Box::new(scroll_area)
                .padding(params! {PaddingParams::uniform(10.0)}, world)
                .card(
                    params! {CardParams {
                        background_color: Color::BLACK.with_alpha(0.3),
                        border_size: 2.0,
                        border_color: Color::WHITE.with_alpha(0.5),
                        corner_radius: 5.0,
                    }},
                    world,
                );
            tileset_preview_elements.push(Box::new(preview_card));
        }
    }
    (
        Box::new(
            Flex::new_column_unweighted(tileset_preview_elements, false, world)
                .padding(params! {PaddingParams::uniform(10.0) }, world)
                .card(
                    params! {CardParams {
                        background_color: Color::BLACK.with_alpha(0.3),
                        border_size: 2.0,
                        border_color: Color::WHITE.with_alpha(0.5),
                        corner_radius: 5.0,
                    }},
                    world,
                ),
        ),
        scroll_area_id,
    )
}

pub fn build_fps_counter<ApplicationEvent: 'static>(
    metrics: Rc<dyn TextMetrics>,
    world: &mut ElementWorld,
) -> Box<dyn Element<State = Editor<ApplicationEvent>, Message = EditorEvent>> {
    let counter_text = TextField::new(
        move |args: StateToParamsArgs<'_, Editor<ApplicationEvent>>| TextFieldParams {
            visuals: TextVisuals {
                text: format!(
                    "FPS: {:7>.2}",
                    args.state
                        .fps
                        .borrow()
                        .current_fps
                        .borrow()
                        .as_ref()
                        .unwrap_or(&0.0)
                ),
                caret_position: None,
                font: UI_FONT.to_string(),
                font_size: UI_FONT_SIZE,
                color: Color::WHITE.with_alpha(0.3),
                caret_width: None,
                caret_blink_duration: None,
            },
            metrics: metrics.clone(),
            interaction: Default::default(),
            text_field_action_handler: EventHandler::none(),
        },
        world,
    )
    .card(
        params! {CardParams {
            background_color: Color::BLACK.with_alpha(0.3),
            border_size: 2.0,
            border_color: Color::WHITE.with_alpha(0.5),
            corner_radius: 5.0,
        }},
        world,
    );
    Box::new(counter_text)
}
