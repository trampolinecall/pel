use std::marker::PhantomData;

use crate::visualizer::{
    event, graphics, layout,
    render_object::{animated::Animated, RenderObject, RenderObjectId, RenderObjectIdMaker},
    widgets::Widget,
};

pub(crate) struct Padding<Data, Child: Widget<Data>> {
    child: Child,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,

    _phantom: PhantomData<fn(&mut Data)>,
}

pub(crate) struct PaddingRenderObject<Data, Child: RenderObject<Data>> {
    child: Child,
    size: graphics::Vector2f,
    left: Animated<f32>,
    top: Animated<f32>,
    right: Animated<f32>,
    bottom: Animated<f32>,

    _phantom: PhantomData<fn(&mut Data)>,
}

impl<Data, Child: Widget<Data>> Padding<Data, Child> {
    pub(crate) fn new(child: Child, left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self { child, left, top, right, bottom, _phantom: PhantomData }
    }
    pub(crate) fn all_around(child: Child, pad: f32) -> Self {
        Self::new(child, pad, pad, pad, pad)
    }
}

impl<Data, Child: Widget<Data>> Widget<Data> for Padding<Data, Child> {
    type Result = PaddingRenderObject<Data, <Child as Widget<Data>>::Result>;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        PaddingRenderObject {
            child: self.child.to_render_object(id_maker),
            size: graphics::Vector2f::new(0.0, 0.0),
            left: Animated::new(self.left),
            top: Animated::new(self.top),
            right: Animated::new(self.right),
            bottom: Animated::new(self.bottom),
            _phantom: PhantomData,
        }
    }

    fn update_render_object(self, render_object: &mut Self::Result, id_maker: &mut RenderObjectIdMaker) {
        self.child.update_render_object(&mut render_object.child, id_maker);
        render_object.left.set(self.left);
        render_object.right.set(self.right);
        render_object.top.set(self.top);
        render_object.bottom.set(self.bottom);
    }
}

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
