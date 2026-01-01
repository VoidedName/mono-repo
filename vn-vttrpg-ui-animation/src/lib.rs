use std::cell::RefCell;
use std::time::{Duration, Instant};

pub trait Interpolatable {
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

pub trait DerivedInterpolatable: Interpolatable + Clone {}

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

#[derive(Debug, Clone, Copy, Default)]
pub enum Easing {
    #[default]
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
}

impl Easing {
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseInQuad => t * t,
            Easing::EaseOutQuad => t * (2.0 - t),
            Easing::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
        }
    }
}

pub struct AnimationState<T> {
    pub start_value: T,
    pub target_value: T,
    pub start_time: Instant,
    pub duration: Duration,
    pub easing: Easing,
}

pub struct AnimationController<T> {
    state: RefCell<AnimationState<T>>,
}

impl<T: Interpolatable + Copy> AnimationController<T> {
    pub fn new(initial_value: T) -> Self {
        Self {
            state: RefCell::new(AnimationState {
                start_value: initial_value,
                target_value: initial_value,
                start_time: Instant::now(),
                duration: Duration::from_secs(0),
                easing: Easing::Linear,
            }),
        }
    }

    pub fn value(&self, now: Instant) -> T {
        let state = self.state.borrow();
        if state.duration.as_secs_f32() == 0.0 {
            return state.target_value;
        }

        let elapsed = now.duration_since(state.start_time);
        let progress = (elapsed.as_secs_f32() / state.duration.as_secs_f32()).clamp(0.0, 1.0);
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
}
