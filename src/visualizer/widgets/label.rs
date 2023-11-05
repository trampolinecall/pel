use sfml::graphics::{Font, Transformable};

use crate::visualizer::{
    event, graphics, layout,
    render_object::{util, RenderObject, RenderObjectId, RenderObjectIdMaker},
    widgets::Widget,
};

// ideally this would just have a reference to the Font object but rust doenst support higher kinded type parameters so i couldnt get it to work
pub(crate) struct Label<GetFont: Fn(&graphics::Fonts) -> &Font> {
    text: String,
    get_font: GetFont,
    font_size: u32,
}
pub(crate) struct LabelRenderObject<GetFont: Fn(&graphics::Fonts) -> &Font> {
    id: RenderObjectId,
    text: String,
    get_font: GetFont,
    font_size: u32,
    size: graphics::Vector2f,
    _private: (),
}

impl<GetFont: Fn(&graphics::Fonts) -> &Font> Label<GetFont> {
    pub(crate) fn new(text: String, get_font: GetFont, font_size: u32) -> Label<GetFont> {
        Label { text, get_font, font_size }
    }
}

impl<GetFont: Fn(&graphics::Fonts) -> &Font, Data> Widget<Data> for Label<GetFont> {
    type Result = LabelRenderObject<GetFont>;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        LabelRenderObject { id: id_maker.next_id(), text: self.text, get_font: self.get_font, font_size: self.font_size, size: graphics::Vector2f::new(0.0, 0.0), _private: () }
    }

    fn update_render_object(self, render_object: &mut Self::Result, _: &mut RenderObjectIdMaker) {
        render_object.text = self.text;
        render_object.get_font = self.get_font;
        render_object.font_size = self.font_size;
    }
}

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
