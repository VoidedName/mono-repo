use crate::{ElementId, ElementImpl, ElementSize, SizeConstraints, StateToParams, UiContext};
use vn_scene::{Color, ImagePrimitiveData, Rect, Scene, TextureId, Transform};
use vn_ui_animation_macros::Interpolatable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitStrategy {
    /// Clip the texture if it's larger than the available space.
    Clip,
    /// Stretch or shrink the texture to fit the available space exactly.
    Stretch,
    /// Shrink or grow the texture to fit within the available space while preserving aspect ratio.
    PreserveAspectRatio,
}

#[derive(Clone, Interpolatable)]
pub struct TextureParams {
    pub texture_id: TextureId,
    pub preferred_size: ElementSize,
    /// NDC coordinates.
    pub uv_rect: Rect,
    pub tint: Color,
    #[interpolate_snappy = "snap_middle"]
    pub fit_strategy: FitStrategy,
}

pub struct Texture<State> {
    id: ElementId,
    params: StateToParams<State, TextureParams>,
}

impl<State> Texture<State> {
    pub fn new(params: StateToParams<State, TextureParams>, ctx: &mut UiContext) -> Self {
        Self {
            id: ctx.event_manager.borrow_mut().next_id(),
            params,
        }
    }
}

impl<State> ElementImpl for Texture<State> {
    type State = State;

    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        constraints: SizeConstraints,
    ) -> ElementSize {
        let params = (self.params)(state, &ctx.now, self.id);

        let size = match params.fit_strategy {
            FitStrategy::Clip => params.preferred_size,
            FitStrategy::Stretch => {
                match (constraints.max_size.width, constraints.max_size.height) {
                    (Some(max_width), Some(max_height)) => ElementSize {
                        width: max_width,
                        height: max_height,
                    },
                    (Some(max_width), None) => ElementSize {
                        width: max_width,
                        height: params.preferred_size.height,
                    },
                    (None, Some(max_height)) => ElementSize {
                        width: params.preferred_size.width,
                        height: max_height,
                    },
                    (None, None) => params.preferred_size,
                }
            }
            FitStrategy::PreserveAspectRatio => {
                let aspect_ratio = params.preferred_size.width / params.preferred_size.height;
                match (constraints.max_size.width, constraints.max_size.height) {
                    (Some(max_width), Some(max_height)) => {
                        if max_width / max_height > aspect_ratio {
                            ElementSize {
                                width: max_height * aspect_ratio,
                                height: max_height,
                            }
                        } else {
                            ElementSize {
                                width: max_width,
                                height: max_width / aspect_ratio,
                            }
                        }
                    }
                    (Some(max_width), None) => ElementSize {
                        width: max_width,
                        height: max_width / aspect_ratio,
                    },
                    (None, Some(max_height)) => ElementSize {
                        width: max_height * aspect_ratio,
                        height: max_height,
                    },
                    (None, None) => params.preferred_size,
                }
            }
        };

        size.clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
    ) {
        let params = (self.params)(state, &ctx.now, self.id);

        let (w, h) = match params.fit_strategy {
            FitStrategy::Clip => (params.preferred_size.width, params.preferred_size.height),
            FitStrategy::Stretch | FitStrategy::PreserveAspectRatio => (size.width, size.height),
        };

        canvas.add_image(ImagePrimitiveData {
            transform: Transform {
                translation: [origin.0, origin.1],
                ..Transform::DEFAULT
            },
            size: [w, h],
            tint: params.tint,
            texture_id: params.texture_id,
            uv_rect: params.uv_rect,
            clip_rect: Rect {
                position: [0.0, 0.0],
                size: [size.width, size.height],
            },
        });
    }
}
