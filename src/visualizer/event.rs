use crate::visualizer::graphics;

#[derive(Copy, Clone)]
pub(crate) enum TargetedEvent {
    LeftMouseDown(graphics::Vector2f),
    RightMouseDown(graphics::Vector2f),
}
#[derive(Copy, Clone)]
pub(crate) enum GeneralEvent {
    MouseMoved(graphics::Vector2f),
    LeftMouseUp,
}
