use crate::visualizer::{
    event, graphics, layout,
    render_object::{RenderObject, RenderObjectId, RenderObjectIdMaker},
    widgets::Widget,
};

pub(crate) struct Empty;

pub(crate) struct EmptyRenderObject {
    size: graphics::Vector2f,
}

impl<Data> Widget<Data> for Empty {
    type Result = EmptyRenderObject;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        EmptyRenderObject { size: graphics::Vector2f::new(0.0, 0.0) }
    }

    fn update_render_object(self, render_object: &mut Self::Result, id_maker: &mut RenderObjectIdMaker) {}
}

impl<Data> RenderObject<Data> for EmptyRenderObject {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        self.size = sc.clamp_size(graphics::Vector2f::new(0.0, 0.0));
    }

    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>) {}

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId> {
        None
    }

    fn size(&self) -> graphics::Vector2f {
        self.size
    }

    fn send_targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, target: RenderObjectId, event: event::TargetedEvent) {}
    fn targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, event: event::TargetedEvent) {}
    fn general_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, event: event::GeneralEvent) {}
}
