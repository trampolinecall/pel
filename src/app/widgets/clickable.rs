use std::marker::PhantomData;

// TODO: REMOVE this whole module (let dom handle click events)

use wasm_bindgen::JsCast;

use crate::app::{vdom, widgets::Widget};

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum MouseButton {
    Main,
    Secondary,
}
pub(crate) struct Clickable<Data, Child: Widget<Data>, Callback: Fn(&mut Data)> {
    mouse_button: MouseButton,
    on_click: Callback,
    child: Child,

    _phantom: PhantomData<fn(&mut Data)>,
}

/* TODO: REMOVE
pub(crate) struct ClickableRenderObject<Data, NormalChild: RenderObject<Data>, ChildOnClicked: RenderObject<Data>, Callback: Fn(&mut Data)> {
    id: RenderObjectId,
    mouse_button: MouseButton,
    on_click: Callback,
    normal_child: NormalChild,
    child_on_clicked: ChildOnClicked,

    clicked: bool,

    _phantom: PhantomData<fn(&mut Data)>,
    _private: (),
}
*/

impl<Data, Child: Widget<Data>, Callback: Fn(&mut Data)> Clickable<Data, Child, Callback> {
    pub(crate) fn new(mouse_button: MouseButton, on_click: Callback, normal_child: Child) -> Self {
        Self { mouse_button, on_click, child: normal_child, _phantom: PhantomData }
    }
}

impl<Data, Child: Widget<Data>, Callback: Fn(&mut Data) + 'static> Widget<Data> for Clickable<Data, Child, Callback> {
    fn to_vdom(self) -> vdom::Element<Data> {
        let mut child = self.child.to_vdom();
        let button_number_looking_for = match self.mouse_button {
            MouseButton::Main => 0,
            MouseButton::Secondary => 2,
        };
        child.event_listeners.push((
            "click",
            Box::new(move |event, data| {
                if event.dyn_ref::<web_sys::PointerEvent>().expect("click event received data that is not PointerEvent").button() == button_number_looking_for {
                    (self.on_click)(data);
                }
            }),
        ));
        child
    }
}

/* TODO: REMOVE
impl<Data, NormalChild: RenderObject<Data>, ChildOnClicked: RenderObject<Data>, Callback: Fn(&mut Data)> RenderObject<Data> for ClickableRenderObject<Data, NormalChild, ChildOnClicked, Callback> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        self.normal_child.layout(graphics_context, sc);
        self.child_on_clicked.layout(graphics_context, sc);
    }

    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>) {
        if self.clicked {
            self.child_on_clicked.draw(graphics_context, target, top_left, hover);
        } else {
            self.normal_child.draw(graphics_context, target, top_left, hover);
        }
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId> {
        if graphics::FloatRect::from_vecs(top_left, self.size()).contains(mouse) {
            Some(self.id)
        } else if self.clicked {
            self.child_on_clicked.find_hover(top_left, mouse)
        } else {
            self.normal_child.find_hover(top_left, mouse)
        }
    }

    fn size(&self) -> graphics::Vector2f {
        if self.clicked {
            self.child_on_clicked.size()
        } else {
            self.normal_child.size()
        }
    }

    fn send_targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, target: RenderObjectId, event: event::TargetedEvent) {
        if target == self.id {
            self.targeted_event(top_left, data, event);
        }

        if self.clicked {
            self.child_on_clicked.send_targeted_event(top_left, data, target, event);
        } else {
            self.normal_child.send_targeted_event(top_left, data, target, event);
        }
    }

    fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, event: event::TargetedEvent) {
        match event {
            event::TargetedEvent::LeftMouseDown(_) => {
                if self.mouse_button == MouseButton::Left {
                    self.clicked = true;
                }
            }
            event::TargetedEvent::RightMouseDown(_) => {
                if self.mouse_button == MouseButton::Right {
                    self.clicked = true;
                }
            }
        }
    }
    fn general_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, event: event::GeneralEvent) {
        match event {
            event::GeneralEvent::MouseMoved(new_mouse_pos) => {
                if !graphics::FloatRect::from_vecs(top_left, self.size()).contains(new_mouse_pos) {
                    self.clicked = false;
                }
            }
            event::GeneralEvent::LeftMouseUp => {
                if self.mouse_button == MouseButton::Left && self.clicked {
                    self.clicked = false;
                    (self.on_click)(data);
                }
            }
            event::GeneralEvent::RightMouseUp => {
                if self.mouse_button == MouseButton::Right && self.clicked {
                    self.clicked = false;
                    (self.on_click)(data);
                }
            }
            event::GeneralEvent::KeyPressed { .. } => {}
        }

        if self.clicked {
            self.child_on_clicked.general_event(top_left, data, event);
        } else {
            self.normal_child.general_event(top_left, data, event);
        }
    }
}
*/
