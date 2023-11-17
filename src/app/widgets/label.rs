use std::collections::HashMap;

use crate::app::{vdom, widgets::Widget};

// ideally this would just have a reference to the Font object but rust doenst support higher kinded type parameters so i couldnt get it to work
pub(crate) struct Label {
    text: String,
    font: String,
    font_size: u32,
}
/* TODO: REMOVE
pub(crate) struct LabelRenderObject<GetFont: Fn(&graphics::Fonts) -> &Font> {
    id: RenderObjectId,
    text: String,
    get_font: GetFont,
    font_size: u32,
    size: graphics::Vector2f,
    _private: (),
}
*/

impl Label {
    pub(crate) fn new(text: String, font: String, font_size: u32) -> Label {
        Label { text, font, font_size }
    }
}

impl<Data> Widget<Data> for Label {
    fn to_vdom(self) -> vdom::Element<Data> {
        // TODO: font, font size
        vdom::Element { type_: vdom::ElementType::P, props: HashMap::new(), event_listeners: vec![], children: vec![vdom::Node::Text(self.text)] }
    }
}

/* TODO: REMOVE
impl<GetFont: Fn(&graphics::Fonts) -> &Font, Data> RenderObject<Data> for LabelRenderObject<GetFont> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        let text = graphics::Text::new(&self.text, (self.get_font)(&graphics_context.fonts), self.font_size);
        let global_bounds = text.global_bounds();
        self.size = sc.clamp_size(graphics::Vector2f::new(global_bounds.left + global_bounds.width, global_bounds.top + global_bounds.height));
    }

    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, _: Option<RenderObjectId>) {
        // TODO: deal with overflow better than by clipping
        // TODO: also fix messy rendering that is caused by clipping
        util::clip(graphics_context, target, graphics::FloatRect::from_vecs(top_left, self.size), |target, top_left| {
            let mut text = graphics::Text::new(&self.text, (self.get_font)(&graphics_context.fonts), self.font_size);
            text.set_position(top_left);
            text.set_fill_color(graphics::Color::WHITE); // TODO: control text color
            target.draw(&text);
        });
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId> {
        if graphics::FloatRect::from_vecs(top_left, self.size).contains(mouse) {
            Some(self.id)
        } else {
            None
        }
    }

    fn size(&self) -> graphics::Vector2f {
        self.size
    }

    fn send_targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, target: RenderObjectId, event: event::TargetedEvent) {
        if target == self.id {
            self.targeted_event(top_left, data, event);
        }
    }

    fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::TargetedEvent) {}
    fn general_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::GeneralEvent) {}
}
*/
