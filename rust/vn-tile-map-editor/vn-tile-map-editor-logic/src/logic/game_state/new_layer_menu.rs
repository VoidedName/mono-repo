use crate::logic::game_state::{ApplicationStateEx, ListParams, btn, label, list, with_fps};
use crate::logic::{ApplicationContext, ApplicationEvent};
use crate::{UI_FONT, UI_FONT_SIZE};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::Color;
use vn_ui::{AnchorExt, ButtonAction, CardExt, CardParams, Element, ElementWorld, Empty, EventHandler, EventManager, Flex, FlexChild, FlexDirection, FlexParams, PaddingExt, PaddingParams, PreferSizeExt, PreferSizeParams, ScrollAreaAction, ScrollAreaExt, ScrollAreaParams, ScrollBarParams, center, params, Stack};

pub struct NewLayerState {
    existing_tileset_names: Vec<String>,
    selected_tileset: Option<usize>,
    scroll_x: ScrollBarParams,
    scroll_y: ScrollBarParams,
    error: Option<String>,
}

#[derive(Clone, Debug)]
pub enum NewLayerEvent {
    New,
    UseSelected,
    Cancel,
    SelectLayer(usize),
    ScrollX(f32),
    ScrollY(f32),
}

pub struct NewLayerMenu {
    #[allow(unused)]
    ui: RefCell<Box<dyn Element<State = NewLayerState, Message = NewLayerEvent>>>,
    #[allow(unused)]
    state: NewLayerState,
    #[allow(unused)]
    ctx: ApplicationContext,
    event_manager: Rc<RefCell<EventManager>>,
}

impl NewLayerMenu {
    pub fn set_error(&mut self, error: String) {
        self.state.error = Some(error)
    }
}

impl NewLayerMenu {
    pub fn new(existing_tileset_names: Vec<String>, ctx: ApplicationContext) -> Self {
        let world = Rc::new(RefCell::new(ElementWorld::new()));

        let title = label(
            |_| "Selecting Tileset for Layer".to_string(),
            UI_FONT,
            UI_FONT_SIZE,
            Color::WHITE,
            ctx.text_metrics.clone(),
            world.clone(),
        )
        .padding(params!(PaddingParams::bottom(25.0)), world.clone())
        .anchor(center!(), world.clone());

        let new = btn(
            |_| "Load New".to_string(),
            UI_FONT,
            UI_FONT_SIZE,
            |_: &NewLayerState| false,
            |_| Color::WHITE,
            |_| Color::WHITE,
            |_| Color::WHITE,
            ctx.text_metrics.clone(),
            EventHandler::new(|_, e| match e {
                ButtonAction::Clicked => vec![NewLayerEvent::New],
            }),
            world.clone(),
        );

        let use_selected = btn(
            |_| "Use Selected".to_string(),
            UI_FONT,
            UI_FONT_SIZE,
            |state: &NewLayerState| state.selected_tileset.is_none(),
            |_| Color::WHITE,
            |_| Color::WHITE,
            |_| Color::WHITE,
            ctx.text_metrics.clone(),
            EventHandler::new(|_, e| match e {
                ButtonAction::Clicked => vec![NewLayerEvent::UseSelected],
            }),
            world.clone(),
        );

        let cancel = btn(
            |_| "Cancel".to_string(),
            UI_FONT,
            UI_FONT_SIZE,
            |_: &NewLayerState| false,
            |_| Color::WHITE,
            |_| Color::WHITE,
            |_| Color::WHITE,
            ctx.text_metrics.clone(),
            EventHandler::new(|_, e| match e {
                ButtonAction::Clicked => vec![NewLayerEvent::Cancel],
            }),
            world.clone(),
        );

        let list = list(
            {
                let mut children: Vec<Rc<RefCell<FlexChild<NewLayerState, NewLayerEvent>>>> =
                    vec![];

                for idx in 0..existing_tileset_names.len() {
                    children.push(Rc::new(RefCell::new(FlexChild::new(btn(
                        move |state: &NewLayerState| state.existing_tileset_names[idx].clone(),
                        UI_FONT,
                        UI_FONT_SIZE,
                        |_| false,
                        move |state: &NewLayerState| {
                            if state
                                .selected_tileset
                                .map(|s_idx| s_idx == idx)
                                .unwrap_or(false)
                            {
                                Color::GREEN
                            } else {
                                Color::WHITE.with_alpha(0.8)
                            }
                        },
                        |_| Color::TRANSPARENT,
                        |_| Color::WHITE,
                        ctx.text_metrics.clone(),
                        EventHandler::new(move |_, e| match e {
                            ButtonAction::Clicked => vec![NewLayerEvent::SelectLayer(idx)],
                        }),
                        world.clone(),
                    )))));
                }

                move |a: &NewLayerState| ListParams {
                    len: a.existing_tileset_names.len(),
                    child: Box::new({
                        let children = children.clone();
                        move |_, idx, _| children[idx].clone()
                    }),
                }
            },
            FlexDirection::Column,
            true,
            world.clone(),
        )
        .scroll_area(
            params!( args<NewLayerState> =>
                ScrollAreaParams {
                    scroll_x: args.state.scroll_x,
                    scroll_y: args.state.scroll_y,
                    scroll_action_handler: EventHandler::new(|_, e| {
                        match e {
                            ScrollAreaAction::ScrollX(v) => vec![NewLayerEvent::ScrollX(v)],
                            ScrollAreaAction::ScrollY(v) => vec![NewLayerEvent::ScrollY(v)],
                        }
                    })
                }
            ),
            world.clone(),
        )
        .prefer_size(
            params!(PreferSizeParams {
                width: None,
                height: Some(400.0),
            }),
            world.clone(),
        )
        .padding(params!(PaddingParams::uniform(10.0)), world.clone())
        .card(
            params!(CardParams {
                border_color: Color::WHITE,
                corner_radius: 5.0,
                border_size: 2.0,
                background_color: Color::BLACK,
            }),
            world.clone(),
        );

        let error = label(
            |state: &NewLayerState| state.error.as_ref().unwrap_or(&"".to_string()).clone(),
            UI_FONT,
            UI_FONT_SIZE,
            Color::RED,
            ctx.text_metrics.clone(),
            world.clone(),
        );

        let layout = Flex::new(
            {
                let children = vec![
                    FlexChild::new(title).into_rc_refcell(),
                    FlexChild::new(list).into_rc_refcell(),
                    FlexChild::new(
                        Empty::new(world.clone())
                            .padding(params!(PaddingParams::vertical(25.0)), world.clone()),
                    )
                    .into_rc_refcell(),
                    FlexChild::new(
                        error.padding(params!(PaddingParams::bottom(25.0)), world.clone()),
                    )
                    .into_rc_refcell(),
                    FlexChild::new(
                        Flex::new(
                            {
                                let children = vec![
                                    FlexChild::new(new).into_rc_refcell(),
                                    FlexChild::new(Empty::new(world.clone()).padding(
                                        params!(PaddingParams::horizontal(10.0)),
                                        world.clone(),
                                    ))
                                    .into_rc_refcell(),
                                    FlexChild::new(use_selected).into_rc_refcell(),
                                    FlexChild::new(Empty::new(world.clone()).padding(
                                        params!(PaddingParams::horizontal(10.0)),
                                        world.clone(),
                                    ))
                                    .into_rc_refcell(),
                                    FlexChild::new(cancel).into_rc_refcell(),
                                ];
                                params!(FlexParams {
                                    children: children.clone(),
                                    force_orthogonal_same_size: true,
                                    direction: FlexDirection::Row
                                })
                            },
                            world.clone(),
                        )
                        .anchor(center!(), world.clone()),
                    )
                    .into_rc_refcell(),
                ];
                params!(FlexParams {
                    children: children.clone(),
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
                background_color: Color::BLACK,
                corner_radius: 5.0,
            }),
            world.clone(),
        )
        .anchor(center!(), world.clone());

        let scroll_bar = ScrollBarParams {
            width: 16.0,
            margin: 8.0,
            color: Color::WHITE,
            position: Some(0.0),
        };

        Self {
            ui: RefCell::new(with_fps(&ctx, Box::new(layout), world.clone())),
            state: NewLayerState {
                existing_tileset_names,
                selected_tileset: None,
                scroll_x: scroll_bar,
                scroll_y: scroll_bar,
                error: None,
            },
            ctx,
            event_manager: Rc::new(RefCell::new(EventManager::new())),
        }
    }
}

impl ApplicationStateEx for NewLayerMenu {
    type StateEvent = NewLayerEvent;
    type State = NewLayerState;
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
            NewLayerEvent::New => Some(ApplicationEvent::LoadTileset(
                self.state.existing_tileset_names.clone(),
            )),
            NewLayerEvent::UseSelected => Some(ApplicationEvent::TilesetReuse(
                self.state.existing_tileset_names[self.state.selected_tileset.unwrap()].clone(),
            )),
            NewLayerEvent::Cancel => Some(ApplicationEvent::TilesetLoadCanceled),
            NewLayerEvent::SelectLayer(idx) => {
                self.state.selected_tileset = Some(idx);
                None
            }
            NewLayerEvent::ScrollX(v) => {
                self.state.scroll_x.position = Some(v);
                None
            }
            NewLayerEvent::ScrollY(v) => {
                self.state.scroll_y.position = Some(v);
                None
            }
        }
    }
}
