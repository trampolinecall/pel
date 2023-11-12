use std::marker::PhantomData;

// TODO: REMOVE this whole widget?

use crate::visualizer::{
    widgets::Widget, dom, graphics
};

pub(crate) struct MinSize<Data, Child: Widget<Data>> {
    child: Child,
    min_size: graphics::Vector2f,

    _phantom: PhantomData<fn(&mut Data)>,
}

/* TODO: REMOVE
pub(crate) struct MinSizeRenderObject<Data, Child: RenderObject<Data>> {
    child: Child,
    min_size: graphics::Vector2f,

    _phantom: PhantomData<fn(&mut Data)>,
}
*/

impl<Data, Child: Widget<Data>> MinSize<Data, Child> {
    pub(crate) fn new(child: Child, min_size: graphics::Vector2f) -> Self {
        Self { child, min_size, _phantom: PhantomData }
    }
}

impl<Data, Child: Widget<Data>> Widget<Data> for MinSize<Data, Child> {
    fn to_vdom(self) -> dom::Element<Data> {
        todo!()
    }
}

/* TODO: REMOVE
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

    fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::TargetedEvent) {}
    fn general_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::GeneralEvent) {}
}
*/
