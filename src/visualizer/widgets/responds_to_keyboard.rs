// TODO: this should probably be removed when i figure out a better event dispatch system
use std::marker::PhantomData;

use crate::visualizer::{
    event, graphics, layout,
    render_object::{RenderObject, RenderObjectId, RenderObjectIdMaker},
    widgets::Widget,
};

pub(crate) struct RespondsToKeyboard<Data, Child: Widget<Data>, Callback: Fn(&mut Data)> {
    key: sfml::window::Key,
    on_press: Callback,
    child: Child,

    _phantom: PhantomData<fn(&mut Data)>,
}

pub(crate) struct RespondsToKeyboardRenderObject<Data, Child: RenderObject<Data>, Callback: Fn(&mut Data)> {
    id: RenderObjectId,
    key: sfml::window::Key,
    on_press: Callback,
    child: Child,

    _phantom: PhantomData<fn(&mut Data)>,
    _private: (),
}

impl<Data, Child: Widget<Data>, Callback: Fn(&mut Data)> RespondsToKeyboard<Data, Child, Callback> {
    pub(crate) fn new(mouse_button: sfml::window::Key, on_press: Callback, child: Child) -> Self {
        Self { key: mouse_button, on_press, child, _phantom: PhantomData }
    }
}

impl<Data, Child: Widget<Data>, Callback: Fn(&mut Data)> Widget<Data> for RespondsToKeyboard<Data, Child, Callback> {
    type Result = RespondsToKeyboardRenderObject<Data, <Child as Widget<Data>>::Result, Callback>;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        RespondsToKeyboardRenderObject { id: id_maker.next_id(), key: self.key, on_press: self.on_press, child: self.child.to_render_object(id_maker), _phantom: PhantomData, _private: () }
    }

    fn update_render_object(self, render_object: &mut Self::Result, id_maker: &mut RenderObjectIdMaker) {
        render_object.key = self.key;
        render_object.on_press = self.on_press;
        self.child.update_render_object(&mut render_object.child, id_maker);
    }
}

impl<Data, Child: RenderObject<Data>, Callback: Fn(&mut Data)> RenderObject<Data> for RespondsToKeyboardRenderObject<Data, Child, Callback> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        self.child.layout(graphics_context, sc);
    }

    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>) {
        self.child.draw(graphics_context, target, top_left, hover);
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId> {
        if graphics::FloatRect::from_vecs(top_left, self.size()).contains(mouse) {
            Some(self.id)
        } else {
            self.child.find_hover(top_left, mouse)
        }
    }

    fn size(&self) -> graphics::Vector2f {
        self.child.size()
    }

    fn send_targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, target: RenderObjectId, event: event::TargetedEvent) {
        if target == self.id {
            self.targeted_event(top_left, data, event);
        }

        self.child.send_targeted_event(top_left, data, target, event);
    }

    fn targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, event: event::TargetedEvent) {}
    fn general_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, event: event::GeneralEvent) {
        match event {
            event::GeneralEvent::MouseMoved(_) => {}
            event::GeneralEvent::LeftMouseUp => {}
            event::GeneralEvent::RightMouseUp => {}
            event::GeneralEvent::KeyPressed { code, alt, ctrl, shift, system } => {
                if code == self.key {
                    // TODO: modifier keys?
                    (self.on_press)(data)
                }
            }
        }

        self.child.general_event(top_left, data, event);
    }
}
