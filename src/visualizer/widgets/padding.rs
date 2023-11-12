use std::marker::PhantomData;

use crate::visualizer::{
    widgets::Widget, vdom
};

// TODO: REMOVE this whole module?

pub(crate) struct Padding<Data, Child: Widget<Data>> {
    child: Child,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,

    _phantom: PhantomData<fn(&mut Data)>,
}

/* TODO: REMOVE
pub(crate) struct PaddingRenderObject<Data, Child: RenderObject<Data>> {
    child: Child,
    size: graphics::Vector2f,
    left: Animated<f32>,
    top: Animated<f32>,
    right: Animated<f32>,
    bottom: Animated<f32>,

    _phantom: PhantomData<fn(&mut Data)>,
}
*/

impl<Data, Child: Widget<Data>> Padding<Data, Child> {
    pub(crate) fn new(child: Child, left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self { child, left, top, right, bottom, _phantom: PhantomData }
    }
    pub(crate) fn all_around(child: Child, pad: f32) -> Self {
        Self::new(child, pad, pad, pad, pad)
    }
}

impl<Data, Child: Widget<Data>> Widget<Data> for Padding<Data, Child> {
    fn to_vdom(self) -> vdom::Element<Data> {
        todo!()
    }
}

/* TODO: REMOVE
impl<Data, Child: RenderObject<Data>> RenderObject<Data> for PaddingRenderObject<Data, Child> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        let shrunk_sc = sc.shrink(graphics::Vector2f::new(self.left.get_lerped() + self.right.get_lerped(), self.top.get_lerped() + self.bottom.get_lerped()));
        self.child.layout(graphics_context, shrunk_sc);
        self.size = sc.clamp_size(self.child.size() + graphics::Vector2f::new(self.left.get_lerped() + self.right.get_lerped(), self.top.get_lerped() + self.bottom.get_lerped()));
    }

    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>) {
        // TODO: calculate offset better in order to account for cases where the padding must be cut off because it would be too big to fit in the size constraints
        self.child.draw(graphics_context, target, top_left + graphics::Vector2f::new(self.left.get_lerped(), self.top.get_lerped()), hover);
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId> {
        self.child.find_hover(top_left + graphics::Vector2f::new(self.left.get_lerped(), self.top.get_lerped()), mouse)
    }

    fn size(&self) -> graphics::Vector2f {
        self.size
    }

    fn send_targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, target: RenderObjectId, event: event::TargetedEvent) {
        self.child.send_targeted_event(top_left + graphics::Vector2f::new(self.left.get_lerped(), self.top.get_lerped()), data, target, event);
    }

    fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::TargetedEvent) {}
    fn general_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::GeneralEvent) {}
}
*/
