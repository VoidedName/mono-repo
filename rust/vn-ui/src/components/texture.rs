use crate::{
    ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints, StateToParams, UiContext,
};
use vn_scene::{Color, ImagePrimitiveData, Rect, Scene, TextureId, Transform};
use vn_ui_animation::Interpolatable;
use vn_ui_animation_macros::Interpolatable;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FitStrategy {
    /// Clip the texture if it's larger than the available space.
    Clip { rotation: f32 },
    /// Stretch or shrink the texture to fit the available space exactly.
    Stretch,
    /// Shrink or grow the texture to fit within the available space while preserving aspect ratio.
    PreserveAspectRatio { rotation: f32 },
}

// eh... do i even want to?
impl Interpolatable for FitStrategy {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        match (self, other) {
            (
                FitStrategy::Clip {
                    rotation: rotation1,
                },
                FitStrategy::Clip {
                    rotation: rotation2,
                },
            ) => FitStrategy::Clip {
                rotation: rotation1.interpolate(rotation2, t),
            },
            (
                FitStrategy::PreserveAspectRatio {
                    rotation: rotation1,
                },
                FitStrategy::PreserveAspectRatio {
                    rotation: rotation2,
                },
            ) => FitStrategy::PreserveAspectRatio {
                rotation: rotation1.interpolate(rotation2, t),
            },
            (FitStrategy::Stretch, FitStrategy::Stretch) => FitStrategy::Stretch,
            _ => {
                if t > 0.5 {
                    other.clone()
                } else {
                    self.clone()
                }
            }
        }
    }
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
    pub fn new(params: StateToParams<State, TextureParams>, world: &mut ElementWorld) -> Self {
        Self {
            id: world.next_id(),
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
        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        let (size, rotation) = match params.fit_strategy {
            FitStrategy::Clip { rotation } => (params.preferred_size, rotation),
            FitStrategy::Stretch => {
                let size = match (constraints.max_size.width, constraints.max_size.height) {
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
                };
                (size, 0.0)
            }
            FitStrategy::PreserveAspectRatio { rotation } => {
                let cos = rotation.cos().abs();
                let sin = rotation.sin().abs();
                let aspect_ratio = params.preferred_size.width / params.preferred_size.height;
                let size = match (constraints.max_size.width, constraints.max_size.height) {
                    (Some(max_width), Some(max_height)) => {
                        // W = w*cos + h*sin = h*R*cos + h*sin = h*(R*cos + sin)
                        // H = w*sin + h*cos = h*R*sin + h*cos = h*(R*sin + cos)
                        let h1 = max_width / (aspect_ratio * cos + sin);
                        let h2 = max_height / (aspect_ratio * sin + cos);
                        let h = h1.min(h2);
                        ElementSize {
                            width: h * aspect_ratio,
                            height: h,
                        }
                    }
                    (Some(max_width), None) => {
                        let h = max_width / (aspect_ratio * cos + sin);
                        ElementSize {
                            width: h * aspect_ratio,
                            height: h,
                        }
                    }
                    (None, Some(max_height)) => {
                        let h = max_height / (aspect_ratio * sin + cos);
                        ElementSize {
                            width: h * aspect_ratio,
                            height: h,
                        }
                    }
                    (None, None) => params.preferred_size,
                };
                (size, rotation)
            }
        };

        let size = size.rotate(rotation);

        ElementSize {
            width: size.width.max(constraints.min_size.width),
            height: size.height.max(constraints.min_size.height),
        }
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
    ) {
        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        let (w, h, rotation) = match params.fit_strategy {
            FitStrategy::Clip { rotation } => (
                params.preferred_size.width,
                params.preferred_size.height,
                rotation,
            ),
            FitStrategy::Stretch => (size.width, size.height, 0.0),
            FitStrategy::PreserveAspectRatio { rotation } => {
                let cos = rotation.cos().abs();
                let sin = rotation.sin().abs();
                let aspect_ratio = params.preferred_size.width / params.preferred_size.height;
                let h1 = size.width / (aspect_ratio * cos + sin);
                let h2 = size.height / (aspect_ratio * sin + cos);
                let h = h1.min(h2);
                (h * aspect_ratio, h, rotation)
            }
        };

        canvas.add_image(ImagePrimitiveData {
            transform: Transform {
                translation: [origin.0 + size.width / 2.0, origin.1 + size.height / 2.0],
                origin: [0.5, 0.5],
                rotation,
                ..Transform::DEFAULT
            },
            size: [w, h],
            tint: params.tint,
            texture_id: params.texture_id,
            uv_rect: params.uv_rect,
            clip_rect: Rect {
                position: [origin.0, origin.1],
                size: [size.width, size.height],
            },
        });
    }
}
