use sfml::graphics::Shape;

use crate::visualizer::{
    event::{GeneralEvent, TargetedEvent},
    graphics, layout,
    render_object::{RenderObject, RenderObjectId, RenderObjectIdMaker},
    widgets::Widget,
};

pub(crate) struct TestRect {
    color: graphics::Color,
    size: graphics::Vector2f,
}
pub(crate) struct TestRectRenderObject {
    id: RenderObjectId,
    color: graphics::Color,
    ideal_size: graphics::Vector2f,
    layout_size: graphics::Vector2f,
    _private: (),
}

impl TestRect {
    pub(crate) fn new(color: graphics::Color, size: graphics::Vector2f) -> TestRect {
        TestRect { color, size }
    }
}

impl<Data> Widget<Data> for TestRect {
    type Result = TestRectRenderObject;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        TestRectRenderObject { color: self.color, ideal_size: self.size, _private: (), layout_size: graphics::Vector2f::new(0.0, 0.0), id: id_maker.next_id() }
    }

    fn update_render_object(self, render_object: &mut Self::Result, _: &mut RenderObjectIdMaker) {
        // TODO: animate?
        render_object.color = self.color;
        render_object.ideal_size = self.size;
    }
}

impl<Data> RenderObject<Data> for TestRectRenderObject {
    fn layout(&mut self, _: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        self.layout_size = sc.clamp_size(self.ideal_size);
    }

    fn draw(&self, _: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>) {
        let rect = graphics::FloatRect::from_vecs(top_left, self.layout_size);
        let mut rect_shape = graphics::RectangleShape::from_rect(rect);
        rect_shape.set_fill_color(self.color);

        if hover == Some(self.id) {
            rect_shape.set_outline_color(graphics::Color::rgba(255, 255, 255, 100)); // TODO: pick a better color for this
            rect_shape.set_outline_thickness(5.0);
        }

        target.draw(&rect_shape);
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId> {
        if graphics::FloatRect::from_vecs(top_left, self.layout_size).contains(mouse) {
            Some(self.id)
        } else {
            None
        }
    }

    fn size(&self) -> graphics::Vector2f {
        self.layout_size
    }

    fn send_targeted_event(&self, data: &mut Data, target: RenderObjectId, event: TargetedEvent) {
        if target == self.id {
            self.targeted_event(data, event);
        }
    }

    fn targeted_event(&self, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&self, _: &mut Data, _: GeneralEvent) {}
}
