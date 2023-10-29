pub(crate) mod util;

use crate::visualizer::{event, graphics, layout};

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct RenderObjectId(u64);

pub(crate) struct RenderObjectIdMaker(u64);
impl RenderObjectIdMaker {
    pub(crate) fn new() -> RenderObjectIdMaker {
        RenderObjectIdMaker(0)
    }
    pub(crate) fn next_id(&mut self) -> RenderObjectId {
        let id = RenderObjectId(self.0);
        self.0 += 1;
        id
    }
}

pub(crate) trait RenderObject<Data: ?Sized> {
    // TODO: automate send_targeted_event and find_hover by having iter_children_by_z method?

    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints);
    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>);
    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId>;
    fn size(&self) -> graphics::Vector2f;

    fn send_targeted_event(&self, data: &mut Data, target: RenderObjectId, event: event::TargetedEvent);
    fn targeted_event(&self, data: &mut Data, event: event::TargetedEvent);
    fn general_event(&self, data: &mut Data, event: event::GeneralEvent);
}
