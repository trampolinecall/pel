use std::marker::PhantomData;

use crate::visualizer::{
    event::{GeneralEvent, TargetedEvent},
    graphics, layout,
    render_object::{RenderObject, RenderObjectId, RenderObjectIdMaker},
    widgets::Widget,
};

pub(crate) struct VSplit<Data, Left: Widget<Data>, Right: Widget<Data>> {
    left: Left,
    right: Right,

    _phantom: PhantomData<fn(&mut Data)>,
}

pub(crate) struct VSplitRenderObject<Data, Left: RenderObject<Data>, Right: RenderObject<Data>> {
    left: Left,
    right: Right,

    size: graphics::Vector2f,

    _phantom: PhantomData<fn(&mut Data)>,
    _private: (),
}

impl<Data, Left: Widget<Data>, Right: Widget<Data>> VSplit<Data, Left, Right> {
    pub(crate) fn new(left: Left, right: Right) -> Self {
        Self { left, right, _phantom: PhantomData }
    }
}

impl<Data, Left: Widget<Data>, Right: Widget<Data>> Widget<Data> for VSplit<Data, Left, Right> {
    type Result = VSplitRenderObject<Data, <Left as Widget<Data>>::Result, <Right as Widget<Data>>::Result>;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        VSplitRenderObject {
            left: self.left.to_render_object(id_maker),
            right: self.right.to_render_object(id_maker),
            size: graphics::Vector2f::new(0.0, 0.0),
            _phantom: PhantomData,
            _private: (),
        }
    }

    fn update_render_object(self, render_object: &mut Self::Result, id_maker: &mut RenderObjectIdMaker) {
        self.left.update_render_object(&mut render_object.left, id_maker);
        self.right.update_render_object(&mut render_object.right, id_maker);
    }
}
impl<Data, Left: RenderObject<Data>, Right: RenderObject<Data>> RenderObject<Data> for VSplitRenderObject<Data, Left, Right> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        let half_sc = layout::SizeConstraints { min: graphics::Vector2f::new(sc.min.x / 2.0, sc.min.y), max: graphics::Vector2f::new(sc.max.x / 2.0, sc.max.y) };
        self.left.layout(graphics_context, half_sc);
        self.right.layout(graphics_context, half_sc);
        self.size = sc.clamp_size(self.left.size() + self.right.size());
    }

    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>) {
        self.left.draw(graphics_context, target, top_left, hover);
        self.right.draw(graphics_context, target, top_left + graphics::Vector2f::new(self.left.size().x, 0.0), hover);
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId> {
        self.left.find_hover(top_left, mouse).or_else(|| self.right.find_hover(top_left + graphics::Vector2f::new(self.left.size().x, 0.0), mouse))
    }

    fn size(&self) -> graphics::Vector2f {
        self.size
    }

    fn send_targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, target: RenderObjectId, event: TargetedEvent) {
        self.left.send_targeted_event(top_left, data, target, event);
        self.right.send_targeted_event(top_left, data, target, event);
    }

    fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: GeneralEvent) {}
}
