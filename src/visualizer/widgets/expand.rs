use std::marker::PhantomData;

use crate::visualizer::{
    event, graphics, layout,
    render_object::{RenderObject, RenderObjectId, RenderObjectIdMaker},
    widgets::Widget,
};

pub(crate) struct Expand<Data, Child: Widget<Data>> {
    child: Child,

    _phantom: PhantomData<fn(&mut Data)>,
}

pub(crate) struct ExpandRenderObject<Data, Child: RenderObject<Data>> {
    child: Child,

    _phantom: PhantomData<fn(&mut Data)>,
}

impl<Data, Child: Widget<Data>> Expand<Data, Child> {
    pub(crate) fn new(child: Child) -> Self {
        Self { child, _phantom: PhantomData }
    }
}

impl<Data, Child: Widget<Data>> Widget<Data> for Expand<Data, Child> {
    type Result = ExpandRenderObject<Data, <Child as Widget<Data>>::Result>;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        ExpandRenderObject { child: self.child.to_render_object(id_maker), _phantom: PhantomData}
    }

    fn update_render_object(self, render_object: &mut Self::Result, id_maker: &mut RenderObjectIdMaker) {
        self.child.update_render_object(&mut render_object.child, id_maker);
    }
}

impl<Data, Child: RenderObject<Data>> RenderObject<Data> for ExpandRenderObject<Data, Child> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        self.child.layout(graphics_context, layout::SizeConstraints { min: sc.max, max: sc.max });
    }

    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>) {
        self.child.draw(graphics_context, target, top_left, hover);
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId> {
        self.child.find_hover(top_left, mouse)
    }

    fn size(&self) -> graphics::Vector2f {
        self.child.size()
    }

    fn send_targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, target: RenderObjectId, event: event::TargetedEvent) {
        self.child.send_targeted_event(top_left, data, target, event);
    }

    fn targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, event: event::TargetedEvent) {}
    fn general_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, event: event::GeneralEvent) {}
}

