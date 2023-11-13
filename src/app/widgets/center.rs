use std::marker::PhantomData;

use crate::app::{vdom, widgets::Widget};

pub(crate) struct Center<Data, Child: Widget<Data>> {
    child: Child,

    _phantom: PhantomData<fn(&mut Data)>,
}

/*
pub(crate) struct CenterRenderObject<Data, Child: RenderObject<Data>> {
    child: Child,
    size: graphics::Vector2f,

    _phantom: PhantomData<fn(&mut Data)>,
}
*/

impl<Data, Child: Widget<Data>> Center<Data, Child> {
    pub(crate) fn new(child: Child) -> Self {
        Self { child, _phantom: PhantomData }
    }
}

impl<Data, Child: Widget<Data>> Widget<Data> for Center<Data, Child> {
    fn to_vdom(self) -> vdom::Element<Data> {
        vdom::Element {
            type_: vdom::ElementType::Div,
            // TODO: do something about .into_iter().collect() (and remember to change it across the whole project)
            props: vec![("style".to_string(), "width: 100%; height: 100%; display: flex; justify-content: center; align-items: center;".into())].into_iter().collect(),
            event_listeners: Vec::new(),
            children: vec![vdom::Node::Element(self.child.to_vdom())],
        }
    }
}

/* TODO: REMOVE
impl<Data, Child: RenderObject<Data>> RenderObject<Data> for CenterRenderObject<Data, Child> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        self.child.layout(graphics_context, sc.with_no_min());
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

    fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::TargetedEvent) {}
    fn general_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::GeneralEvent) {}
}

fn center(top_left: graphics::Vector2f, max_size: graphics::Vector2f, child_size: graphics::Vector2f) -> graphics::Vector2f {
    top_left + max_size * 0.5 - (child_size / 2.0)
}
*/
