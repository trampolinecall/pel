use crate::visualizer::{dom, graphics, widgets::Widget};

pub(crate) struct TestRect {
    color: graphics::Color,
    size: graphics::Vector2f,
}
/* TODO: REMOVE
pub(crate) struct TestRectRenderObject {
    id: RenderObjectId,
    color: graphics::Color,
    ideal_size: graphics::Vector2f,
    layout_size: graphics::Vector2f,
    _private: (),
}
*/

impl TestRect {
    pub(crate) fn new(color: graphics::Color, size: graphics::Vector2f) -> TestRect {
        TestRect { color, size }
    }
}

impl<Data> Widget<Data> for TestRect {
    fn to_vdom(self) -> dom::Element<Data> {
        // TODO: sizes in other units
        dom::Element {
            type_: dom::ElementType::Div,
            props: vec![("style".to_string(), format!("width: {}px; height: {}px; color: {}", self.size.x, self.size.y, self.color.to_css_color()).into())].into_iter().collect(),
            event_listeners: Vec::new(),
            children: Vec::new(),
        }
    }
}

/* TODO: REMOVE
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

    fn send_targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, target: RenderObjectId, event: TargetedEvent) {
        if target == self.id {
            self.targeted_event(top_left, data, event);
        }
    }

    fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: GeneralEvent) {}
}
*/
