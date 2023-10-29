use std::marker::PhantomData;

use crate::visualizer::{
    event, graphics, layout,
    render_object::{RenderObject, RenderObjectId, RenderObjectIdMaker},
    widgets::Widget,
};

pub(crate) struct Center<Data, Child: Widget<Data>> {
    child: Child,

    _phantom: PhantomData<fn(&mut Data)>,
}

pub(crate) struct CenterRenderObject<Data, Child: RenderObject<Data>> {
    child: Child,
    size: graphics::Vector2f,

    clicked: bool,

    _phantom: PhantomData<fn(&mut Data)>,
    _private: (),
}

impl<Data, Child: Widget<Data>> Center<Data, Child> {
    pub(crate) fn new(child: Child) -> Self {
        Self { child, _phantom: PhantomData }
    }
}

impl<Data, Child: Widget<Data>> Widget<Data> for Center<Data, Child> {
    type Result = CenterRenderObject<Data, <Child as Widget<Data>>::Result>;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        CenterRenderObject { child: self.child.to_render_object(id_maker), size: graphics::Vector2f::new(0.0, 0.0), clicked: false, _phantom: PhantomData, _private: () }
    }

    fn update_render_object(self, render_object: &mut Self::Result, id_maker: &mut RenderObjectIdMaker) {
        self.child.update_render_object(&mut render_object.child, id_maker);
    }
}

impl<Data, Child: RenderObject<Data>> RenderObject<Data> for CenterRenderObject<Data, Child> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        self.child.layout(graphics_context, sc);
        self.size = sc.max;
    }

    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>) {
        self.child.draw(graphics_context, target, center(top_left, self.size, self.child.size()), hover);
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId> {
        self.child.find_hover(center(top_left, self.size, self.child.size()), mouse)
    }

    fn size(&self) -> graphics::Vector2f {
        self.size
    }

    fn send_targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, target: RenderObjectId, event: event::TargetedEvent) {
        self.child.send_targeted_event(center(top_left, self.size, self.child.size()), data, target, event);
    }

    fn targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, event: event::TargetedEvent) {}
    fn general_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, event: event::GeneralEvent) {}
}

fn center(top_left: graphics::Vector2f, max_size: graphics::Vector2f, child_size: graphics::Vector2f) -> graphics::Vector2f {
    top_left + max_size * 0.5 - (child_size / 2.0)
}
