use crate::visualizer::{
    graphics,
    widgets::{max_size::MaxSize, min_size::MinSize, Widget},
};

pub(crate) fn fixed_size<Data, Child: Widget<Data>>(child: Child, size: graphics::Vector2f) -> impl Widget<Data> {
    MinSize::new(MaxSize::new(child, size), size)
}
