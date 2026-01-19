use vn_ui_animation_macros::Interpolatable;

#[derive(Clone, Copy, Debug, Default, Interpolatable, PartialEq)]
pub struct InteractionState {
    #[interpolate_snappy = "snap_start"]
    pub is_hovered: bool,
    #[interpolate_snappy = "snap_start"]
    pub is_focused: bool,
}
