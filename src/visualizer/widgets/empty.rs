use crate::visualizer::{vdom, widgets::Widget};

pub(crate) struct Empty;

/* TODO: REMOVE
pub(crate) struct EmptyRenderObject {
    size: graphics::Vector2f,
}
*/

impl<Data> Widget<Data> for Empty {
    fn to_vdom(self) -> vdom::Element<Data> {
        todo!()
    }
}

/* TODO: REMOVE
impl<Data> RenderObject<Data> for EmptyRenderObject {
    fn layout(&mut self, _: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        self.size = sc.clamp_size(graphics::Vector2f::new(0.0, 0.0));
    }

    fn draw(&self, _: &graphics::GraphicsContext, _: &mut dyn graphics::RenderTarget, _: graphics::Vector2f, _: Option<RenderObjectId>) {}

    fn find_hover(&self, _: graphics::Vector2f, _: graphics::Vector2f) -> Option<RenderObjectId> {
        None
    }

    fn size(&self) -> graphics::Vector2f {
        self.size
    }

    fn send_targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: RenderObjectId, _: event::TargetedEvent) {}
    fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::TargetedEvent) {}
    fn general_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::GeneralEvent) {}
}
*/
