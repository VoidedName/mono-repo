use crate::logic::app_state::{
    Brush, EditorEvent, EditorState, Input, TextFieldState, btn, empty_texture, input, label,
    labelled_input, suppress_enter_key,
};
use crate::logic::{ApplicationContext, Grid, GridEvent, GridParams, PlatformHooks};
use crate::{UI_FONT, UI_FONT_SIZE};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use vn_scene::{Color, Rect};
use vn_tilemap::{TileMap, TileMapParams};
use vn_ui::*;
use vn_wgpu_window::resource_manager::Sampling;

pub fn layers<Platform: PlatformHooks>(
    ctx: &ApplicationContext<Platform>,
    world: Rc<RefCell<ElementWorld>>,
) -> Box<dyn Element<State = EditorState, Message = EditorEvent>> {
    let title = label(
        |_| "Layer Settings".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        Color::WHITE,
        ctx.text_metrics.clone(),
        world.clone(),
    )
    .padding(
        params!(PaddingParams {
            pad_bottom: 25.0,
            ..Default::default()
        }),
        world.clone(),
    )
    .anchor(center!(), world.clone());

    let new_layer = btn(
        |_| "Add Layer".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        |_| false,
        |_| Color::WHITE,
        |_| Color::WHITE,
        |_| Color::WHITE,
        ctx.text_metrics.clone(),
        EventHandler::new(|_, e| match e {
            ButtonAction::Clicked => {
                vec![EditorEvent::TryAddingLayer]
            }
        }),
        world.clone(),
    );

    let save = btn(
        |_| "Save".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        |_| false,
        |_| Color::WHITE,
        |_| Color::WHITE,
        |_| Color::WHITE,
        ctx.text_metrics.clone(),
        EventHandler::new(|_, e| match e {
            ButtonAction::Clicked => {
                vec![EditorEvent::SaveSpec]
            }
        }),
        world.clone(),
    );

    let load = btn(
        |_| "Load".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        |_| false,
        |_| Color::WHITE,
        |_| Color::WHITE,
        |_| Color::WHITE,
        ctx.text_metrics.clone(),
        EventHandler::new(|_, e| match e {
            ButtonAction::Clicked => {
                vec![EditorEvent::TryLoadSpec]
            }
        }),
        world.clone(),
    );

    let save_load = Flex::new(
        {
            let c = vec![
                FlexChild::new(save).into_rc_refcell(),
                FlexChild::new(
                    Empty::new(world.clone())
                        .padding(params!(PaddingParams::horizontal(25.0)), world.clone()),
                )
                .into_rc_refcell(),
                FlexChild::new(load).into_rc_refcell(),
            ];
            params!(FlexParams {
                direction: FlexDirection::Row,
                force_orthogonal_same_size: false,
                children: c.clone()
            })
        },
        world.clone(),
    )
    .anchor(bottom_right!(), world.clone());

    let layer_flex = Flex::new(
        {
            let cache: Rc<RefCell<Vec<Rc<RefCell<FlexChild<EditorState, EditorEvent>>>>>> =
                Rc::new(RefCell::new(vec![]));
            let world = world.clone();
            let metrics = ctx.text_metrics.clone();

            params!(args<EditorState> => {
                let cache_len = { cache.borrow().len() };

                for idx in cache_len..args.state.tile_map.layers.len() {
                    let Input { element: layer_name, .. } = input(
                        move |state: &EditorState| TextFieldState {
                            id: ElementId(0),
                            text: state.tile_map.layers[idx].name.clone(),
                            caret: state.layer_caret_positions[idx].clone(),
                        },
                        Some("Layer Name".to_string()),
                        UI_FONT,
                        UI_FONT_SIZE,
                        metrics.clone(),
                        EventHandler::new(move |_, e| match e {
                            TextFieldAction::TextChange(new_name) => vec![EditorEvent::RenameLayer(idx, new_name)],
                            TextFieldAction::CaretMove(caret) => vec![EditorEvent::LayerCaretPosition(idx, caret)],
                        }).with_overwrite(suppress_enter_key()),
                        world.clone(),
                    );

                     let layer = btn(
                        move |state: &EditorState|  if state.current_layer.map(|l| l == idx).unwrap_or(false) {
                            "■".to_string()
                        } else {
                            "■".to_string()
                        },
                        UI_FONT,
                        UI_FONT_SIZE,
                        |_| false,
                        move |state: &EditorState| if state.current_layer.map(|l| l == idx).unwrap_or(false) {
                            Color::GREEN
                        } else {
                            Color::WHITE.with_alpha(0.8)
                        },
                        |_| Color::TRANSPARENT,
                        |_| Color::WHITE,
                        metrics.clone(),
                        EventHandler::new(move |_, e| match e {
                            ButtonAction::Clicked => vec![EditorEvent::SwitchToLayer(idx)],
                        }),
                        world.clone(),
                    );

                    let layer_up = btn(
                        move |_| "▲".to_string(),
                        UI_FONT,
                        UI_FONT_SIZE,
                        move |state: &EditorState| idx + 1 == state.tile_map.layers.len(),
                        |_| Color::WHITE,
                        |_| Color::TRANSPARENT,
                        |_| Color::WHITE,
                        metrics.clone(),
                        EventHandler::new(move |_, e| match e {
                            ButtonAction::Clicked => vec![EditorEvent::MoveLayer(idx, idx + 1)],
                        }),
                        world.clone(),
                    );

                    let layer_down = btn(
                        move |_| "▼".to_string(),
                        UI_FONT,
                        UI_FONT_SIZE,
                        move |_| idx == 0,
                        |_| Color::WHITE,
                        |_| Color::TRANSPARENT,
                        |_| Color::WHITE,
                        metrics.clone(),
                        EventHandler::new(move |_, e| match e {
                            ButtonAction::Clicked => vec![EditorEvent::MoveLayer(idx, idx - 1)],
                        }),
                        world.clone(),
                    );

                    let remove = btn(
                        |_| "X".to_string(),
                        UI_FONT,
                        UI_FONT_SIZE,
                        |_| false,
                        |_| Color::RED,
                        |_| Color::TRANSPARENT,
                        |_| Color::RED,
                        metrics.clone(),
                        EventHandler::new(move |_, e| match e {
                            ButtonAction::Clicked => vec![EditorEvent::DeleteLayer(idx)],
                        }),
                        world.clone(),
                    );

                    let layout = Flex::new({
                        let layer = FlexChild::new(layer).into_rc_refcell();
                        let layer_name = FlexChild::weighted(layer_name, 1.0).into_rc_refcell();
                        let remove = FlexChild::new(remove).into_rc_refcell();
                        let layer_up = FlexChild::new(layer_up).into_rc_refcell();
                        let layer_down = FlexChild::new(layer_down).into_rc_refcell();
                        params!(FlexParams {
                            direction: FlexDirection::Row,
                            force_orthogonal_same_size: false,
                            children: vec![layer.clone(), layer_up.clone(), layer_down.clone(), layer_name.clone(), remove.clone()],
                        })
                    }, world.clone());

                    cache.borrow_mut().push(FlexChild::new(
                        layout.padding(params!(PaddingParams::vertical(10.0)), world.clone()),
                    ).into_rc_refcell())
                }

                FlexParams {
                    direction: FlexDirection::Column,
                    children: cache.borrow()[0..args.state.tile_map.layers.len()].iter().rev().cloned().collect(),
                    force_orthogonal_same_size: true
            }})
        },
        world.clone(),
    );

    let layer_list = layer_flex
        .padding(params!(PaddingParams::uniform(5.0)), world.clone())
        .card(
            params!(CardParams {
                border_color: Color::WHITE,
                corner_radius: 5.0,
                border_size: 2.0,
                background_color: Color::BLACK,
            }),
            world.clone(),
        );

    Flex::new(
        {
            let c = vec![
                FlexChild::new(Flex::new(
                    {
                        let c = vec![
                            FlexChild::weighted(Empty::new(world.clone()), 1.0).into_rc_refcell(),
                        ];
                        params!(FlexParams {
                            force_orthogonal_same_size: true,
                            direction: FlexDirection::Row,
                            children: c.clone()
                        })
                    },
                    world.clone(),
                ))
                .into_rc_refcell(),
                FlexChild::new(title).into_rc_refcell(),
                FlexChild::new(
                    layer_list.padding(params!(PaddingParams::bottom(25.0)), world.clone()),
                )
                .into_rc_refcell(),
                FlexChild::new(new_layer).into_rc_refcell(),
                FlexChild::weighted(Empty::new(world.clone()), 1.0).into_rc_refcell(),
                FlexChild::new(save_load).into_rc_refcell(),
            ];
            params!(FlexParams {
                direction: FlexDirection::Column,
                children: c.clone(),
                force_orthogonal_same_size: true,
            })
        },
        world.clone(),
    )
    .padding(params!(PaddingParams::uniform(25.0)), world.clone())
    .anchor(top!(), world.clone())
    .fill(world.clone())
    .card(
        params!(CardParams {
            border_color: Color::WHITE,
            border_size: 2.0,
            background_color: Color::BLACK,
            corner_radius: 5.0,
        }),
        world.clone(),
    )
    .prefer_size(
        params!(PreferSizeParams {
            width: Some(400.0),
            height: None,
        }),
        world.clone(),
    )
    .into()
}

pub fn editor<Platform: PlatformHooks>(
    ctx: &ApplicationContext<Platform>,
    world: Rc<RefCell<ElementWorld>>,
) -> Box<dyn Element<State = EditorState, Message = EditorEvent>> {
    let title = label(
        |_| "Map".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        Color::WHITE,
        ctx.text_metrics.clone(),
        world.clone(),
    )
    .padding(
        params!(PaddingParams {
            pad_bottom: 25.0,
            ..Default::default()
        }),
        world.clone(),
    )
    .anchor(center!(), world.clone());

    let mouse_pos: Rc<RefCell<Option<(u32, u32)>>> = Rc::new(RefCell::new(None));
    let brushing: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
    let grid = Grid::new(
        params!(args<EditorState> =>

        let brushing = brushing.clone();
        let mouse_pos = mouse_pos.clone();

        GridParams {
            cols: args.state.tile_map.map_dimensions.0,
            rows: args.state.tile_map.map_dimensions.1,
            grid_color: Color::WHITE.with_alpha(0.5),
            grid_width: 3.0,
            grid_size: (32.0, 32.0),
            child: Box::new(|_, _, _, _| None),
            event_handler: EventHandler::new(move |_, e| match e {
                    GridEvent::MouseOverCell(x, y) => {
                        let result = if let Some(&(old_x, old_y)) = mouse_pos.borrow().as_ref()
                            && *brushing.borrow()
                        {
                            if old_x != x || old_y != y {
                                vec![EditorEvent::Brushing(x, y)]
                            } else {
                                vec![]
                            }
                        } else {
                            vec![]
                        };

                        mouse_pos.borrow_mut().replace((x, y));
                        result
                    }
                    GridEvent::MouseDown(btn) => {
                        if let Some(&(x, y)) = mouse_pos.borrow().as_ref()
                            && btn == MouseButton::Left
                        {
                            *brushing.borrow_mut() = true;
                            vec![EditorEvent::Brushing(x, y)]
                        } else {
                            vec![]
                        }
                    }
                    GridEvent::MouseUp(btn) => {
                        if btn == MouseButton::Left {
                            *brushing.borrow_mut() = false;
                        }
                        vec![]
                    }
                }),
            }),
        world.clone(),
    );

    let map = TileMap::new(
        params!(args<EditorState> => TileMapParams {
            draw_tile_size: ElementSize {
                width: 32.0,
                height: 32.0,
            },
            textures: args.state.tile_map.layers.iter()
                .map(|l| args.state.loaded_tilesets.get(&l.tileset).unwrap().texture_id.clone())
                .collect(),
            specification: args.state.tile_map.clone(),
        }),
        world.clone(),
    );

    let map = Stack::new(vec![map.into(), grid.into()], world.clone())
        .anchor(center!(), world.clone())
        .scroll_area(
            params!(args<EditorState> => ScrollAreaParams {
                scroll_x: args.state.tilemap_view_scroll_x,
                scroll_y: args.state.tilemap_view_scroll_y,
                scroll_action_handler: EventHandler::new(|_, e| match e {
                    ScrollAreaAction::ScrollX(v) => vec![EditorEvent::TilemapViewScrollX(v)],
                    ScrollAreaAction::ScrollY(v) => vec![EditorEvent::TilemapViewScrollY(v)],
                })
            }),
            world.clone(),
        )
        .fill(world.clone());

    let eraser_brush = btn(
        |_| "Eraser Brush".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        |_| false,
        |_| Color::WHITE,
        |_| Color::WHITE,
        |_| Color::WHITE,
        ctx.text_metrics.clone(),
        EventHandler::new(|_, e| match e {
            ButtonAction::Clicked => {
                vec![EditorEvent::TileBrushSelect(Brush::Eraser)]
            }
        }),
        world.clone(),
    );

    let clear_brush = btn(
        |_| "Clear Brush".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        |_| false,
        |_| Color::WHITE,
        |_| Color::WHITE,
        |_| Color::WHITE,
        ctx.text_metrics.clone(),
        EventHandler::new(|_, e| match e {
            ButtonAction::Clicked => {
                vec![EditorEvent::TileBrushSelect(Brush::None)]
            }
        }),
        world.clone(),
    );

    let current_brush = label(
        |s: &EditorState| {
            format!(
                "Brush: {}",
                match &s.brush {
                    Brush::None => {
                        "None   ".to_string()
                    }
                    Brush::Eraser => {
                        "Eraser ".to_string()
                    }
                    Brush::Tileset(_, _) => {
                        "Tileset".to_string()
                    }
                }
            )
        },
        UI_FONT,
        UI_FONT_SIZE,
        Color::WHITE,
        ctx.text_metrics.clone(),
        world.clone(),
    );

    let current_tilemap_size = label(
        |s: &EditorState| {
            format!(
                "Current Tilemap Dimensions:\n  {:?}",
                s.tile_map.map_dimensions
            )
        },
        UI_FONT,
        UI_FONT_SIZE,
        Color::WHITE,
        ctx.text_metrics.clone(),
        world.clone(),
    );

    let tilemap_width_controller = Rc::new(RefCell::new(TextFieldState {
        id: ElementId(0),
        text: "10".to_string(),
        caret: Some(0),
    }));
    let tilemap_width_setting = Rc::new(RefCell::new(10));

    let tilemap_height_controller = Rc::new(RefCell::new(TextFieldState {
        id: ElementId(0),
        text: "5".to_string(),
        caret: Some(0),
    }));
    let tilemap_height_setting = Rc::new(RefCell::new(5));

    let apply_settings = btn(
        |_| "Apply Settings".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        {
            let tilemap_height_setting = tilemap_height_setting.clone();
            let tilemap_width_setting = tilemap_width_setting.clone();
            move |s: &EditorState| {
                let w = tilemap_width_setting.borrow().clone();
                let h = tilemap_height_setting.borrow().clone();
                (w == 0 || h == 0)
                    || (s.tile_map.map_dimensions.0 == w && s.tile_map.map_dimensions.1 == h)
            }
        },
        |_| Color::WHITE,
        |_| Color::WHITE,
        |_| Color::WHITE,
        ctx.text_metrics.clone(),
        EventHandler::new({
            let tilemap_width_setting = tilemap_width_setting.clone();
            let tilemap_height_setting = tilemap_height_setting.clone();

            move |_, e| match e {
                ButtonAction::Clicked => {
                    vec![EditorEvent::ChangeTilemapSize(
                        tilemap_width_setting.clone().borrow().clone(),
                        tilemap_height_setting.clone().borrow().clone(),
                    )]
                }
            }
        }),
        world.clone(),
    );

    let tilemap_width = labelled_input(
        {
            let tilemap_width_controller = tilemap_width_controller.clone();
            move |_| tilemap_width_controller.clone().borrow().clone()
        },
        "Tilemap Width: ",
        UI_FONT,
        UI_FONT_SIZE,
        ctx.text_metrics.clone(),
        EventHandler::new({
            let tilemap_width_setting = tilemap_width_setting.clone();
            move |_, e| match e {
                TextFieldAction::TextChange(new_text) => {
                    let new_text = new_text.trim().to_string();
                    if new_text.is_empty() {
                        *tilemap_width_setting.clone().borrow_mut() = 0;
                        tilemap_width_controller.clone().borrow_mut().text = new_text;
                    } else {
                        let wide = new_text.parse::<u32>();
                        match wide {
                            Ok(wide) => {
                                *tilemap_width_setting.clone().borrow_mut() = wide;
                                tilemap_width_controller.clone().borrow_mut().text = new_text;
                            }
                            Err(_) => {}
                        }
                    }
                    vec![]
                }
                TextFieldAction::CaretMove(new_caret) => {
                    tilemap_width_controller.borrow_mut().caret = Some(new_caret);
                    vec![]
                }
            }
        })
        .with_overwrite(suppress_enter_key()),
        world.clone(),
    );

    let tilemap_height = labelled_input(
        {
            let tilemap_height_controller = tilemap_height_controller.clone();
            move |_| tilemap_height_controller.clone().borrow().clone()
        },
        "Tilemap Height:",
        UI_FONT,
        UI_FONT_SIZE,
        ctx.text_metrics.clone(),
        EventHandler::new({
            let tilemap_height_setting = tilemap_height_setting.clone();
            move |_, e| match e {
                TextFieldAction::TextChange(new_text) => {
                    let new_text = new_text.trim().to_string();
                    if new_text.is_empty() {
                        *tilemap_height_setting.clone().borrow_mut() = 0;
                        tilemap_height_controller.clone().borrow_mut().text = new_text;
                    } else {
                        let wide = new_text.parse::<u32>();
                        match wide {
                            Ok(wide) => {
                                *tilemap_height_setting.clone().borrow_mut() = wide;
                                tilemap_height_controller.clone().borrow_mut().text = new_text;
                            }
                            Err(_) => {}
                        }
                    }
                    vec![]
                }
                TextFieldAction::CaretMove(new_caret) => {
                    tilemap_height_controller.borrow_mut().caret = Some(new_caret);
                    vec![]
                }
            }
        })
        .with_overwrite(suppress_enter_key()),
        world.clone(),
    );

    let brushes = Flex::new(
        {
            let c = Rc::new(RefCell::new(vec![
                FlexChild::new(current_brush).into_rc_refcell(),
                FlexChild::new(
                    Empty::new(world.clone())
                        .padding(params!(PaddingParams::vertical(10.0)), world.clone()),
                )
                .into_rc_refcell(),
                FlexChild::new(eraser_brush).into_rc_refcell(),
                FlexChild::new(
                    Empty::new(world.clone())
                        .padding(params!(PaddingParams::vertical(10.0)), world.clone()),
                )
                .into_rc_refcell(),
                FlexChild::new(clear_brush).into_rc_refcell(),
            ]));

            params!(FlexParams {
                children: c.borrow().clone(),
                direction: FlexDirection::Column,
                force_orthogonal_same_size: true,
            })
        },
        world.clone(),
    )
    .padding(params!(PaddingParams::uniform(25.0)), world.clone())
    .card(
        params!(CardParams {
            border_color: Color::WHITE,
            border_size: 2.0,
            corner_radius: 5.0,
            background_color: Color::BLACK,
        }),
        world.clone(),
    );

    let setting_errors = label(
        {
            let tilemap_width_setting = tilemap_width_setting.clone();
            let tilemap_height_setting = tilemap_height_setting.clone();
            move |_| {
                let mut needs_newline = true;
                let mut error = "".to_string();
                if *tilemap_width_setting.borrow() == 0 {
                    error.push_str("Height must be > 0");
                }
                if *tilemap_height_setting.borrow() == 0 {
                    if &error != "" {
                        error.push('\n');
                        needs_newline = false;
                    }
                    error.push_str("Width must be > 0");
                }
                if needs_newline {
                    error.push('\n');
                }
                error
            }
        },
        UI_FONT,
        UI_FONT_SIZE,
        Color::RED,
        ctx.text_metrics.clone(),
        world.clone(),
    );

    let settings = Flex::new(
        {
            let c = Rc::new(RefCell::new(vec![
                FlexChild::new(current_tilemap_size).into_rc_refcell(),
                FlexChild::new(
                    Empty::new(world.clone())
                        .padding(params!(PaddingParams::vertical(10.0)), world.clone()),
                )
                .into_rc_refcell(),
                FlexChild::new(tilemap_width.element).into_rc_refcell(),
                FlexChild::new(
                    Empty::new(world.clone())
                        .padding(params!(PaddingParams::vertical(10.0)), world.clone()),
                )
                .into_rc_refcell(),
                FlexChild::new(tilemap_height.element).into_rc_refcell(),
                FlexChild::new(
                    Empty::new(world.clone())
                        .padding(params!(PaddingParams::vertical(10.0)), world.clone()),
                )
                .into_rc_refcell(),
                FlexChild::new(setting_errors).into_rc_refcell(),
                FlexChild::new(
                    Empty::new(world.clone())
                        .padding(params!(PaddingParams::vertical(10.0)), world.clone()),
                )
                .into_rc_refcell(),
                FlexChild::new(apply_settings).into_rc_refcell(),
            ]));

            params!(FlexParams {
                children: c.borrow().clone(),
                direction: FlexDirection::Column,
                force_orthogonal_same_size: true,
            })
        },
        world.clone(),
    )
    .padding(params!(PaddingParams::uniform(25.0)), world.clone())
    .card(
        params!(CardParams {
            border_color: Color::WHITE,
            border_size: 2.0,
            corner_radius: 5.0,
            background_color: Color::BLACK,
        }),
        world.clone(),
    )
    .prefer_size(
        params!(PreferSizeParams {
            width: Some(400.0),
            height: None
        }),
        world.clone(),
    );

    let tool_bar = Flex::new(
        {
            let c = Rc::new(RefCell::new(vec![
                FlexChild::weighted(Empty::new(world.clone()), 1.0).into_rc_refcell(),
                FlexChild::new(brushes).into_rc_refcell(),
                FlexChild::new(
                    Empty::new(world.clone())
                        .padding(params!(PaddingParams::horizontal(25.0)), world.clone()),
                )
                .into_rc_refcell(),
                FlexChild::new(settings).into_rc_refcell(),
            ]));
            params!(FlexParams {
                direction: FlexDirection::Row,
                children: c.borrow().clone(),
                force_orthogonal_same_size: true,
            })
        },
        world.clone(),
    )
    .padding(params!(PaddingParams::uniform(25.0)), world.clone())
    .card(
        params!(CardParams {
            border_color: Color::WHITE,
            border_size: 2.0,
            corner_radius: 5.0,
            background_color: Color::BLACK,
        }),
        world.clone(),
    );

    Box::new(
        Flex::new(
            {
                let c = vec![
                    FlexChild::new(Flex::new(
                        {
                            let c = FlexChild::weighted(Empty::new(world.clone()), 1.0)
                                .into_rc_refcell();
                            params!(FlexParams {
                                direction: FlexDirection::Row,
                                children: vec![c.clone()],
                                force_orthogonal_same_size: true,
                            })
                        },
                        world.clone(),
                    ))
                    .into_rc_refcell(),
                    FlexChild::new(title).into_rc_refcell(),
                    FlexChild::weighted(map, 1.0).into_rc_refcell(),
                    FlexChild::new(tool_bar).into_rc_refcell(),
                ];
                params!(FlexParams {
                    children: c.clone(),
                    direction: FlexDirection::Column,
                    force_orthogonal_same_size: true,
                })
            },
            world.clone(),
        )
        .padding(params!(PaddingParams::uniform(25.0)), world.clone())
        .card(
            params!(CardParams {
                border_color: Color::WHITE,
                border_size: 2.0,
                corner_radius: 5.0,
                background_color: Color::BLACK,
            }),
            world.clone(),
        ),
    )
}

pub fn tileset<Platform: PlatformHooks>(
    ctx: &ApplicationContext<Platform>,
    world: Rc<RefCell<ElementWorld>>,
) -> Box<dyn Element<State = EditorState, Message = EditorEvent>> {
    let title = label(
        |_| "Tileset".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        Color::WHITE,
        ctx.text_metrics.clone(),
        world.clone(),
    )
    .padding(
        params!(PaddingParams {
            pad_bottom: 25.0,
            ..Default::default()
        }),
        world.clone(),
    )
    .anchor(center!(), world.clone());

    let empty_text = ctx
        .rm
        .load_texture_from_bytes(empty_texture(), Sampling::Nearest)
        .expect("empty texture");

    let tileset_tex = Texture::new(
        params!(args<EditorState> =>
            let id = args.state.current_layer.map(|layer |
                args.state.loaded_tilesets
                .get(&args.state.tile_map.layers[layer].tileset)
                .unwrap().texture_id.clone())
            .unwrap_or(empty_text.id.clone());

            let size = args.state.current_layer.map(|layer | {
                (
                    args.state.tile_map.layers[layer].tileset_dimensions.0 * args.state.tile_map.layers[layer].tile_dimensions.0,
                    args.state.tile_map.layers[layer].tileset_dimensions.1 * args.state.tile_map.layers[layer].tile_dimensions.1,
                )
            }).unwrap_or((0, 0));

            let size = ElementSize {
                width: size.0 as f32,
                height: size.1 as f32,
            };

            TextureParams {
                texture_id: id,
                preferred_size: size,
                uv_rect: Rect::UNIT,
                tint: Color::WHITE,
                fit_strategy: FitStrategy::Clip { rotation: 0.0 },
            }
        ),
        world.clone(),
    );

    let grid = Grid::new(
        {
            let mouse_pos: Rc<RefCell<Option<(u32, u32)>>> = Rc::new(RefCell::new(None));
            let start_mouse_pos: Rc<RefCell<Option<(u32, u32)>>> = Rc::new(RefCell::new(None));
            let cache = Rc::new(RefCell::new(HashMap::new()));
            let world = world.clone();

            params!(args<EditorState> =>
            let (cols, rows, grid_w, grid_h) = args.state.current_layer.map(|layer| {
            let tileset_dim = args.state.tile_map.layers[layer].tileset_dimensions;
                let tile_dim = args.state.tile_map.layers[layer].tile_dimensions;
                (tileset_dim.0, tileset_dim.1, tile_dim.0, tile_dim.1)
            }).unwrap_or((0, 0, 0, 0));

            let mouse_pos = mouse_pos.clone();
            let start_mouse_pos = start_mouse_pos.clone();
            let world = world.clone();

            let cache = cache.clone();
            GridParams {
                cols,
                rows,
                grid_color: Color::WHITE.with_alpha(0.25),
                grid_width: 3.0,
                grid_size: (grid_w as f32, grid_h as f32),
                child: Box::new(move |_, cell, state: &EditorState, _| {
                    let world = world.clone();
                    let cache = cache.clone();
                    let mut cache_borrow = cache.borrow_mut();
                    if let Brush::Tileset(from, to) = state.brush {
                        if from.0 <= cell.0 && from.1 <= cell.1 && to.0 >= cell.0 && to.1 >= cell.1 {
                            Some(cache_borrow.entry(cell).or_insert_with(||
                                Rc::new(RefCell::new(Empty::new(world.clone()).fill(world.clone()).card(params!(CardParams {
                                    background_color: Color::WHITE.with_alpha(0.5),
                                    corner_radius: 0.0,
                                    border_size: 0.0,
                                    border_color: Color::WHITE,
                                }), world.clone())))
                            ).clone())
                        } else { None }
                    } else { None }
                }),
                event_handler: EventHandler::new(move |_, e| {
                    match e {
                        GridEvent::MouseOverCell(x, y) => {
                            mouse_pos.borrow_mut().replace((x, y));
                            if let Some(&start_mouse_pos) = start_mouse_pos.borrow().as_ref() {
                                vec![EditorEvent::TileBrushSelect(Brush::Tileset(start_mouse_pos, (x, y)))]
                            } else {
                                vec![]
                            }
                        }
                        GridEvent::MouseDown(btn) => {
                            if let Some(&mouse_pos) = mouse_pos.borrow().as_ref() && btn == MouseButton::Left {
                                start_mouse_pos.borrow_mut().replace(mouse_pos);
                                vec![EditorEvent::TileBrushSelect(Brush::Tileset(mouse_pos, mouse_pos))]
                            } else {
                                vec![]
                            }
                        }
                        GridEvent::MouseUp(btn) => {
                            if btn == MouseButton::Left {
                                start_mouse_pos.borrow_mut().take();
                            }
                            vec![]
                        }
                    }
                }),
            })
        },
        world.clone(),
    );

    let layout = Stack::new(vec![tileset_tex.into(), grid.into()], world.clone());

    let tileset = ScrollArea::new(
        layout,
        params!(args < EditorState > => ScrollAreaParams {
            scroll_x: args.state.tileset_view_scroll_x.clone(),
            scroll_y: args.state.tileset_view_scroll_y.clone(),
            scroll_action_handler: EventHandler::new(| _, e | {
                match e {
                    ScrollAreaAction::ScrollX(v) => vec ! [EditorEvent::TilesetViewScrollX(v)],
                    ScrollAreaAction::ScrollY(v) => vec ! [EditorEvent::TilesetViewScrollY(v)],
                }
            })
        }),
        world.clone(),
    );

    Box::new(
        Flex::new(
            {
                let c = vec![
                    FlexChild::new(title).into_rc_refcell(),
                    FlexChild::weighted(tileset, 1.0).into_rc_refcell(),
                ];
                params!(FlexParams {
                    force_orthogonal_same_size: true,
                    direction: FlexDirection::Column,
                    children: c.clone(),
                })
            },
            world.clone(),
        )
        .padding(params!(PaddingParams::uniform(25.0)), world.clone())
        .anchor(top!(), world.clone())
        .fill(world.clone())
        .card(
            params!(CardParams {
                border_color: Color::WHITE,
                corner_radius: 5.0,
                border_size: 2.0,
                background_color: Color::BLACK,
            }),
            world.clone(),
        )
        .prefer_size(
            params!(PreferSizeParams {
                width: Some(400.0),
                height: None,
            }),
            world.clone(),
        ),
    )
}
