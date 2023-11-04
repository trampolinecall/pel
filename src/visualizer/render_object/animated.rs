use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Animated<T> {
    last_changed: Instant,
    current: T,
    last: Option<T>,
}

pub(crate) enum AnimatedValue<'t, T> {
    Steady(&'t T),
    Animating { before: &'t T, after: &'t T, amount: f64 },
}

// TODO: have configurable animation duration
const ANIMATION_DURATION: Duration = Duration::from_millis(200);

// TODO: allow choice between different easing functions
fn ease(x: f64) -> f64 {
    // easing functions stolen from https://gist.github.com/gre/1650294
    // quartic ease in and out
    /*
    if x < 0.5 {
        8.0 * x.powf(4.0)
    } else {
        1.0 - 8.0 * (x - 1.0).powf(4.0)
    }
    */
    // quartic ease out
    1.0 - (x - 1.0).powf(4.0)
}

impl<T> Animated<T> {
    pub(crate) fn new(item: T) -> Self {
        Self { last_changed: Instant::now(), current: item, last: None }
    }

    pub(crate) fn get(&self) -> AnimatedValue<T> {
        if self.last_changed.elapsed() < ANIMATION_DURATION {
            match &self.last {
                Some(last) => AnimatedValue::Animating { before: last, after: &self.current, amount: ease(self.last_changed.elapsed().as_secs_f64() / ANIMATION_DURATION.as_secs_f64()) },
                None => AnimatedValue::Steady(&self.current),
            }
        } else {
            AnimatedValue::Steady(&self.current)
        }
    }

    pub(crate) fn get_current(&self) -> &T {
        &self.current
    }
}

impl<T: PartialEq> Animated<T> {
    pub(crate) fn set(&mut self, new: T) {
        if self.current != new {
            let last = std::mem::replace(&mut self.current, new);
            self.last = Some(last);
            self.last_changed = Instant::now();
        } else {
            self.current = new;
        }
    }
}

impl<T: Lerpable + Copy> Animated<T> {
    pub(crate) fn get_lerped(&self) -> T {
        match self.get() {
            AnimatedValue::Steady(s) => *s,
            AnimatedValue::Animating { before, after, amount } => before.lerp(after, amount),
        }
    }
}

pub(crate) trait Lerpable {
    fn lerp(&self, other: &Self, amount: f64) -> Self;
}

macro_rules! impl_lerpable_for_numeric {
    ($ty:ty) => {
        impl Lerpable for $ty {
            fn lerp(&self, other: &Self, amount: f64) -> Self {
                (*self as f64 + (*other as f64 - *self as f64) * amount) as $ty
            }
        }
    };
}
impl_lerpable_for_numeric!(f32);
impl_lerpable_for_numeric!(f64);
impl_lerpable_for_numeric!(i8);
impl_lerpable_for_numeric!(i16);
impl_lerpable_for_numeric!(i32);
impl_lerpable_for_numeric!(i64);
impl_lerpable_for_numeric!(isize);
impl_lerpable_for_numeric!(u8);
impl_lerpable_for_numeric!(u16);
impl_lerpable_for_numeric!(u32);
impl_lerpable_for_numeric!(u64);
impl_lerpable_for_numeric!(usize);

impl<T: Lerpable> Lerpable for sfml::system::Vector2<T> {
    fn lerp(&self, other: &Self, amount: f64) -> Self {
        Self { x: self.x.lerp(&other.x, amount), y: self.y.lerp(&other.y, amount) }
    }
}
