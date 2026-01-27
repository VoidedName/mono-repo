use vn_scene::Color;
use vn_ui::{center, AnchorExt, Element, ElementWorld, Empty};
use crate::logic::game_state::{label, EditorEvent, EditorState};
use crate::{UI_FONT, UI_FONT_SIZE};
use crate::logic::ApplicationContext;

pub fn layers(ctx: &ApplicationContext, world: &mut ElementWorld) -> Box<dyn Element<State = EditorState, Message = EditorEvent>> {
    let title = label(
        |_| "Layers".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        Color::WHITE,
        ctx.text_metrics.clone(),
        world
    ).anchor(center!(), world);


    title.into()
}

pub fn editor(ctx: &ApplicationContext, world: &mut ElementWorld) -> Box<dyn Element<State = EditorState, Message = EditorEvent>> {
    let title = label(
        |_| "Map".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        Color::WHITE,
        ctx.text_metrics.clone(),
        world
    ).anchor(center!(), world);


    title.into()
}

pub fn tileset(ctx: &ApplicationContext, world: &mut ElementWorld) -> Box<dyn Element<State = EditorState, Message = EditorEvent>> {
    let title = label(
        |_| "Tileset".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        Color::WHITE,
        ctx.text_metrics.clone(),
        world
    ).anchor(center!(), world);


    title.into()
}