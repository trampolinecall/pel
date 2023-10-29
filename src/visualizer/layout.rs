use crate::visualizer::graphics;

#[derive(Copy, Clone, PartialEq)]
pub(crate) struct SizeConstraints {
    pub(crate) min: graphics::Vector2f,
    pub(crate) max: graphics::Vector2f,
}
impl SizeConstraints {
    pub(crate) fn with_no_min(&self) -> SizeConstraints {
        SizeConstraints { min: graphics::Vector2f::new(0.0, 0.0), max: self.max }
    }

    pub(crate) fn clamp_size(&self, size: graphics::Vector2f) -> graphics::Vector2f {
        graphics::Vector2f::new(size.x.clamp(self.min.x, self.max.x), size.y.clamp(self.min.y, self.max.y))
    }
}

