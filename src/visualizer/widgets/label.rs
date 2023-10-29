use crate::visualizer::{
    event, graphics, layout,
    render_object::{RenderObject, RenderObjectId, RenderObjectIdMaker},
    widgets::Widget,
};

pub(crate) struct Label {
    text: String,
}
pub(crate) struct LabelRenderObject {
    id: RenderObjectId,
    text: String,
    size: graphics::Vector2f,
    _private: (),
}

impl Label {
    pub(crate) fn new(text: String) -> Label {
        Label { text }
    }
}

impl<Data> Widget<Data> for Label {
    type Result = LabelRenderObject;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        LabelRenderObject { id: id_maker.next_id(), text: self.text, size: graphics::Vector2f::new(0.0, 0.0), _private: () }
    }

    fn update_render_object(self, render_object: &mut Self::Result) {
        render_object.text = self.text;
    }
}

impl<Data> RenderObject<Data> for LabelRenderObject {
    fn layout(&mut self, sc: layout::SizeConstraints) {
        todo!()
    }

    fn draw_inner(&self, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>) {
        todo!()
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId> {
        todo!()
    }

    fn size(&self) -> graphics::Vector2f {
        todo!()
    }

    fn send_targeted_event(&self, data: &mut Data, target: RenderObjectId, event: event::TargetedEvent) {
        todo!()
    }

    fn targeted_event(&self, data: &mut Data, event: event::TargetedEvent) {
        todo!()
    }

    fn general_event(&self, data: &mut Data, event: event::GeneralEvent) {
        todo!()
    }
}
