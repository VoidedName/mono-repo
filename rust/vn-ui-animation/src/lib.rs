use std::cell::RefCell;
use std::rc::Rc;
use vn_utils::float::NaNTo;
use web_time::{Duration, Instant};

pub trait Interpolatable: Sized + Clone {
    fn interpolate(&self, other: &Self, t: f32) -> Self;
    fn into_animation_controller(self) -> AnimationController<Self> {
        AnimationController::new(self)
    }
}

macro_rules! impl_interpolatable_for_numeric {
    ($tt:ty) => {
        impl Interpolatable for $tt {
            fn interpolate(&self, other: &Self, t: f32) -> Self {
                (*self as f32 + (*other as f32 - *self as f32) * t) as $tt
            }
        }
    };
}

impl_interpolatable_for_numeric!(f32);
impl_interpolatable_for_numeric!(f64);

impl_interpolatable_for_numeric!(u8);
impl_interpolatable_for_numeric!(u16);
impl_interpolatable_for_numeric!(u32);
impl_interpolatable_for_numeric!(u64);
impl_interpolatable_for_numeric!(u128);

impl_interpolatable_for_numeric!(i8);
impl_interpolatable_for_numeric!(i16);
impl_interpolatable_for_numeric!(i32);
impl_interpolatable_for_numeric!(i64);
impl_interpolatable_for_numeric!(i128);

impl Interpolatable for [f32; 2] {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        [
            self[0].interpolate(&other[0], t),
            self[1].interpolate(&other[1], t),
        ]
    }
}

impl Interpolatable for Duration {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Duration::from_millis(self.as_millis().interpolate(&other.as_millis(), t) as u64)
    }
}

/// Easing describes how to interpolate between two values over time.
///
/// It remaps the linear progress to any arbitrary one between 0.0 and 1.0
#[derive(Clone, Default)]
pub enum Easing {
    #[default]
    /// x => x
    Linear,
    /// x => x^2
    EaseInQuad,
    /// x => 1 - (1 - x)^2
    EaseOutQuad,
    /// x => x < 0.5 ? 2 * x^2 : 1 - (-2 * x + 2)^2 / 2
    EaseInOutQuad,
    /// Any custom easing function. The input is guaranteed to be in \[0.0, 1.0].
    /// The output will be clamped to \[0.0, 1.0], so you can return whatever you want.
    ///
    /// NaN will be coerced to 0.0
    Custom(Rc<Box<dyn Fn(f32) -> f32>>),
}

impl std::fmt::Debug for Easing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Easing::Linear => write!(f, "Linear"),
            Easing::EaseInQuad => write!(f, "EaseInQuad"),
            Easing::EaseOutQuad => write!(f, "EaseOutQuad"),
            Easing::EaseInOutQuad => write!(f, "EaseInOutQuad"),
            Easing::Custom(_) => write!(f, "Custom(<function>)"),
        }
    }
}

impl Easing {
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseInQuad => t * t,
            Easing::EaseOutQuad => 1.0 - (1.0 - t).powi(2),
            Easing::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::Custom(easing_fn) => easing_fn(t).clamp(0.0, 1.0).nan_to(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProgressParams {
    pub elapsed: Duration,
    /// This should NEVER be 0 as many progress functions will divide by it.
    pub duration: Duration,
}

/// Progress indicates how to control animation looping behaviour
///
/// It takes in elapsed and duration and produces the progress between 0.0 and 1.0.
///
/// This happens **BEFORE** [Easing] is applied
#[derive(Clone, Default)]
pub enum Progress {
    #[default]
    /// Animation runs exactly once.
    Once,
    /// Animation will loop forever.
    Loop,
    /// Animation will loop x times.
    Repeat(u32),
    /// Animation will loop forever. It reaches the target value at progress 0.5 and then reverses its direction.
    PingPong,
    /// Any custom progress function. The output will be clamped to \[0.0, 1.0], so you can return whatever you want.
    /// [ProgressParams::duration] will never be 0.0
    ///
    /// NaN will be coerced to 0.0
    Custom(Rc<Box<dyn Fn(ProgressParams) -> f32>>),
}

impl std::fmt::Debug for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Progress::*;

        match self {
            Once => write!(f, "Once"),
            Loop => write!(f, "Loop"),
            Repeat(times) => write!(f, "Repeat({})", times),
            PingPong => write!(f, "PingPong"),
            Custom(_) => write!(f, "Custom(<function>)"),
        }
    }
}

impl Progress {
    /// Takes in elapsed and duration and produces the progress 0.0..=1.0].
    pub fn apply(&self, params: ProgressParams) -> f32 {
        match self {
            Progress::Once => params.elapsed.as_secs_f32() / params.duration.as_secs_f32(),
            Progress::Loop => {
                (params.elapsed.as_secs_f32() % params.duration.as_secs_f32())
                    / params.duration.as_secs_f32()
            }
            Progress::Repeat(times) => {
                let times = *times as f32;

                let progress =
                    (params.elapsed.as_secs_f32() / params.duration.as_secs_f32()).min(times);

                if progress > times {
                    1.0
                } else {
                    progress % 1.0
                }
            }
            Progress::PingPong => {
                let progress = (params.elapsed.as_secs_f32() % params.duration.as_secs_f32())
                    / params.duration.as_secs_f32();
                if progress < 0.5 {
                    progress * 2.0
                } else {
                    (1.0 - progress) * 2.0
                }
            }
            Progress::Custom(progress_fn) => progress_fn(params),
        }
        .clamp(0.0, 1.0)
        .nan_to(0.0)
    }
}

pub struct AnimationState<T> {
    pub start_value: T,
    pub target_value: T,
    pub start_time: Instant,
    pub duration: Duration,
    pub easing: Easing,
    pub progress: Progress,
}

pub struct AnimationController<T> {
    state: RefCell<AnimationState<T>>,
}

impl<T: Interpolatable + Clone> AnimationController<T> {
    pub fn new(initial_value: T) -> Self {
        Self {
            state: RefCell::new(AnimationState {
                start_value: initial_value.clone(),
                target_value: initial_value,
                start_time: Instant::now(),
                duration: Duration::from_secs(0),
                easing: Easing::Linear,
                progress: Progress::Once,
            }),
        }
    }

    pub fn value(&self, now: Instant) -> T {
        let state = self.state.borrow();

        // we do NOT want to divide by zero
        if state.duration.as_secs_f32() == 0.0 {
            return state.target_value.clone();
        }

        let progress = state.progress.apply(ProgressParams {
            elapsed: now.duration_since(state.start_time),
            duration: state.duration,
        });

        let eased_progress = state.easing.apply(progress);

        state
            .start_value
            .interpolate(&state.target_value, eased_progress)
    }

    pub fn update_state<F>(&self, f: F)
    where
        F: FnOnce(&mut AnimationState<T>),
    {
        f(&mut self.state.borrow_mut());
    }

    pub fn into_rc(self) -> Rc<Self> {
        Rc::new(self)
    }
}

impl<T: Interpolatable> From<T> for AnimationController<T> {
    fn from(value: T) -> Self {
        value.into_animation_controller()
    }
}