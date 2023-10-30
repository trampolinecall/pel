use std::marker::PhantomData;

use crate::visualizer::{
    event, graphics, layout,
    render_object::{RenderObject, RenderObjectId, RenderObjectIdMaker},
    widgets::Widget,
};

pub(crate) struct MinSize<Data, Child: Widget<Data>> {
    child: Child,
    min_size: graphics::Vector2f,

    _phantom: PhantomData<fn(&mut Data)>,
}

pub(crate) struct MinSizeRenderObject<Data, Child: RenderObject<Data>> {
    child: Child,
    min_size: graphics::Vector2f,

    _phantom: PhantomData<fn(&mut Data)>,
}

impl<Data, Child: Widget<Data>> MinSize<Data, Child> {
    pub(crate) fn new(child: Child, min_size: graphics::Vector2f) -> Self {
        Self { child, min_size, _phantom: PhantomData }
    }
}

impl<Data, Child: Widget<Data>> Widget<Data> for MinSize<Data, Child> {
    type Result = MinSizeRenderObject<Data, <Child as Widget<Data>>::Result>;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        MinSizeRenderObject { child: self.child.to_render_object(id_maker), min_size: self.min_size, _phantom: PhantomData }
    }

    fn update_render_object(self, render_object: &mut Self::Result, id_maker: &mut RenderObjectIdMaker) {
        render_object.min_size = self.min_size;
        self.child.update_render_object(&mut render_object.child, id_maker);
    }
}

impl<Data, Child: RenderObject<Data>> RenderObject<Data> for MinSizeRenderObject<Data, Child> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        let size = sc.clamp_size(self.min_size);
        self.child.layout(graphics_context, layout::SizeConstraints { min: size, max: sc.max });
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
