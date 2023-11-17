use std::marker::PhantomData;

// TODO: REMOVE whole module?

use crate::app::{graphics, vdom, widgets::Widget};

pub(crate) struct MaxSize<Data, Child: Widget<Data>> {
    child: Child,
    max_size: graphics::Vector2f,

    _phantom: PhantomData<fn(&mut Data)>,
}

/* TODO: REMOVE
pub(crate) struct MaxSizeRenderObject<Data, Child: RenderObject<Data>> {
    child: Child,
    max_size: graphics::Vector2f,

    _phantom: PhantomData<fn(&mut Data)>,
}
*/

impl<Data, Child: Widget<Data>> MaxSize<Data, Child> {
    pub(crate) fn new(child: Child, max_size: graphics::Vector2f) -> Self {
        Self { child, max_size, _phantom: PhantomData }
    }
}

impl<Data, Child: Widget<Data>> Widget<Data> for MaxSize<Data, Child> {
    fn to_vdom(self) -> vdom::Element<Data> {
        vdom::Element {
            type_: vdom::ElementType::Div,
            // TODO: do something about .into_iter().collect() (and remember to change it across the whole project)
            props: vec![("style", format!("max-width: {}px; max-height: {}px;", self.max_size.x, self.max_size.y).into())].into_iter().collect(),
            event_listeners: Vec::new(),
            children: vec![vdom::Node::Element(self.child.to_vdom())],
        }
    }
}

/* TODO: REMOVE
impl<Data, Child: RenderObject<Data>> RenderObject<Data> for MaxSizeRenderObject<Data, Child> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        let size = sc.clamp_size(self.max_size);
        self.child.layout(graphics_context, layout::SizeConstraints { min: sc.min, max: size });
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

    fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::TargetedEvent) {}
    fn general_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::GeneralEvent) {}
}
*/
