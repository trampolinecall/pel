use std::marker::PhantomData;

use crate::visualizer::{
    event, graphics, layout,
    render_object::{RenderObject, RenderObjectId, RenderObjectIdMaker},
    widgets::Widget,
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum MouseButton {
    Left,
    Right,
}
pub(crate) struct Clickable<Data, NormalChild: Widget<Data>, ChildOnClicked: Widget<Data>, Callback: Fn(&mut Data)> {
    mouse_button: MouseButton,
    on_click: Callback,
    normal_child: NormalChild,
    child_on_clicked: ChildOnClicked,

    _phantom: PhantomData<fn(&mut Data)>,
}

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

impl<Data, NormalChild: Widget<Data>, ChildOnClicked: Widget<Data>, Callback: Fn(&mut Data)> Clickable<Data, NormalChild, ChildOnClicked, Callback> {
    pub(crate) fn new(mouse_button: MouseButton, on_click: Callback, normal_child: NormalChild, child_on_clicked: ChildOnClicked) -> Self {
        Self { mouse_button, on_click, normal_child, child_on_clicked, _phantom: PhantomData }
    }
}

impl<Data, NormalChild: Widget<Data>, ChildOnClicked: Widget<Data>, Callback: Fn(&mut Data)> Widget<Data> for Clickable<Data, NormalChild, ChildOnClicked, Callback> {
    type Result = ClickableRenderObject<Data, <NormalChild as Widget<Data>>::Result, <ChildOnClicked as Widget<Data>>::Result, Callback>;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        ClickableRenderObject {
            id: id_maker.next_id(),
            mouse_button: self.mouse_button,
            on_click: self.on_click,
            normal_child: self.normal_child.to_render_object(id_maker),
            child_on_clicked: self.child_on_clicked.to_render_object(id_maker),
            clicked: false,
            _phantom: PhantomData,
            _private: (),
        }
    }

    fn update_render_object(self, render_object: &mut Self::Result, id_maker: &mut RenderObjectIdMaker) {
        render_object.mouse_button = self.mouse_button;
        render_object.on_click = self.on_click;
        self.normal_child.update_render_object(&mut render_object.normal_child, id_maker);
        self.child_on_clicked.update_render_object(&mut render_object.child_on_clicked, id_maker);
    }
}

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

    fn targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, event: event::TargetedEvent) {
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
