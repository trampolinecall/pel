// TODO: this should probably be removed when i figure out a better event dispatch system
use std::marker::PhantomData;

use crate::visualizer::{graphics::Key, vdom, widgets::Widget};

pub(crate) struct RespondsToKeyboard<Data, Child: Widget<Data>, Callback: Fn(&mut Data)> {
    key: Key,
    on_press: Callback,
    child: Child,

    _phantom: PhantomData<fn(&mut Data)>,
}

/* TODO: REMOVE
pub(crate) struct RespondsToKeyboardRenderObject<Data, Child: RenderObject<Data>, Callback: Fn(&mut Data)> {
    id: RenderObjectId,
    key: sfml::window::Key,
    on_press: Callback,
    child: Child,

    _phantom: PhantomData<fn(&mut Data)>,
    _private: (),
}
*/

impl<Data, Child: Widget<Data>, Callback: Fn(&mut Data)> RespondsToKeyboard<Data, Child, Callback> {
    pub(crate) fn new(key: Key, on_press: Callback, child: Child) -> Self {
        Self { key, on_press, child, _phantom: PhantomData }
    }
}

impl<Data, Child: Widget<Data>, Callback: Fn(&mut Data)> Widget<Data> for RespondsToKeyboard<Data, Child, Callback> {
    fn to_vdom(self) -> vdom::Element<Data> {
        todo!()
    }
}

/* TODO: REMOVE
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

    fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::TargetedEvent) {}
    fn general_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, event: event::GeneralEvent) {
        match event {
            event::GeneralEvent::MouseMoved(_) => {}
            event::GeneralEvent::LeftMouseUp => {}
            event::GeneralEvent::RightMouseUp => {}
            event::GeneralEvent::KeyPressed { code, .. } => {
                if code == self.key {
                    // TODO: modifier keys?
                    (self.on_press)(data);
                }
            }
        }

        self.child.general_event(top_left, data, event);
    }
}
*/
