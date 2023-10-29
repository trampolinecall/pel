use sfml::graphics::{Transformable, Shape};

use crate::{
    source::Span,
    visualizer::{
        event, graphics, layout,
        render_object::{RenderObject, RenderObjectId, RenderObjectIdMaker},
        widgets::Widget,
    },
};

pub(crate) struct CodeView<'file> {
    span: Span<'file>, // TODO: secondary spans, arrows, ...
}

pub(crate) struct CodeViewRenderObject<'file> {
    id: RenderObjectId,
    span: Span<'file>,
    size: graphics::Vector2f,
    _private: (),
}

impl<'file> CodeView<'file> {
    pub(crate) fn new(span: Span<'file>) -> Self {
        Self { span }
    }
}

impl<'file, Data> Widget<Data> for CodeView<'file> {
    type Result = CodeViewRenderObject<'file>;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        CodeViewRenderObject { id: id_maker.next_id(), size: graphics::Vector2f::new(0.0, 0.0), _private: (), span: self.span }
    }

    fn update_render_object(self, render_object: &mut Self::Result, _: &mut RenderObjectIdMaker) {
        render_object.span = self.span;
    }
}

impl<'file, Data> RenderObject<Data> for CodeViewRenderObject<'file> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        let text = graphics::Text::new(&self.span.file().source, &graphics_context.font, 15); // TODO: control font size
        self.size = sc.clamp_size(text.global_bounds().size());
    }

    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, _: Option<RenderObjectId>) {
        // TODO: deal with overflow (clipping does not work because the bounding box does not include descenders)
        // util::clip(graphics_context, target, graphics::FloatRect::from_vecs(top_left, self.size), |target, top_left| {
        let mut text = graphics::Text::new(&self.span.file().source, &graphics_context.font, 15); // TODO: control font size
        text.set_position(top_left);
        text.set_fill_color(graphics::Color::WHITE); // TODO: control text color
                                                     //
        let highlight_start_pos = text.find_character_pos(self.span.start());
        let highlight_end_pos = text.find_character_pos(self.span.end());

        let mut start_rect = graphics::RectangleShape::from_rect(graphics::FloatRect::from_vecs(highlight_start_pos, graphics::Vector2f::new(10.0, 10.0)));
        let mut end_rect = graphics::RectangleShape::from_rect(graphics::FloatRect::from_vecs(highlight_end_pos, graphics::Vector2f::new(10.0, 10.0)));

        start_rect.set_fill_color(graphics::Color::GREEN);
        end_rect.set_fill_color(graphics::Color::GREEN);

        target.draw(&start_rect);
        target.draw(&end_rect);
        target.draw(&text);
        // });
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

    fn targeted_event(&mut self, top_left: graphics::Vector2f, _: &mut Data, _: event::TargetedEvent) {}
    fn general_event(&mut self, top_left: graphics::Vector2f, _: &mut Data, _: event::GeneralEvent) {}
}
