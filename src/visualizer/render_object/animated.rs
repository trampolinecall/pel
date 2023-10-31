use std::time::{Duration, Instant};

pub(crate) struct Animated<T> {
    last_changed: Instant,
    current: T,
    last: Option<T>,
}

// TODO: have configurable animation duration
const ANIMATION_DURATION: Duration = Duration::from_millis(200);

impl<T> Animated<T> {
    pub(crate) fn new(item: T) -> Self {
        Self { last_changed: Instant::now(), current: item, last: None }
    }

    pub(crate) fn get(&self) -> Result<&T, (&T, &T, f64)> {
        if self.last_changed.elapsed() < ANIMATION_DURATION {
            match &self.last {
                Some(last) => Err((last, &self.current, self.last_changed.elapsed().as_secs_f64() / ANIMATION_DURATION.as_secs_f64())),
                None => Ok(&self.current),
            }
        } else {
            Ok(&self.current)
        }
    }
}

impl<T: Eq> Animated<T> {
    pub(crate) fn update(&mut self, new: T) {
        if self.current != new {
            let last = std::mem::replace(&mut self.current, new);
            self.last = Some(last);
            self.last_changed = Instant::now(); // TODO: remove Instant argument from RenderObject methods?
        } else {
            self.current = new;
        }
    }
}

impl<T: Lerpable + Copy> Animated<T> {
    pub(crate) fn get_lerped(&self) -> T {
        match self.get() {
            Ok(s) => *s,
            Err((start, end, amount)) => start.lerp(end, amount),
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
