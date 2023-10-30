use sfml::graphics::{Shape, Transformable};

use crate::visualizer::widgets::flex;
use crate::{
    source::Span,
    visualizer::{
        event, graphics, layout,
        render_object::{RenderObject, RenderObjectId, RenderObjectIdMaker},
        widgets::Widget,
    },
};

pub(crate) struct LineView<'file> {
    contents: &'file str,
    highlight_start: Option<usize>,
    highlight_end: Option<usize>,
}

pub(crate) struct LineViewRenderObject<'file> {
    id: RenderObjectId,
    contents: &'file str,
    highlight_start: Option<usize>,
    highlight_end: Option<usize>,
    size: graphics::Vector2f,
    _private: (),
}

// TODO: secondary spans, messages, ...
// TODO: scrolling
pub(crate) fn code_view<'file, Data: 'file>(span: Span<'file>) -> impl Widget<Data> + 'file {
    flex::homogeneous::Flow::new(
        flex::Direction::Vertical,
        span.file
            .lines
            .iter()
            .map(|(line_bounds, line_contents)| {
                (
                    flex::ItemSettings::Fixed,
                    LineView {
                        contents: line_contents,
                        // TODO: these should actually be if the span overlaps with the line range in order to handle the middle lines of a multiline span
                        highlight_start: if line_bounds.contains(&span.start) { Some(span.start - line_bounds.start) } else { None },
                        highlight_end: if line_bounds.contains(&(span.end.saturating_sub(1))) { Some(span.end - line_bounds.start) } else { None },
                    },
                )
            })
            .collect(),
    )
}

impl<'file, Data> Widget<Data> for LineView<'file> {
    type Result = LineViewRenderObject<'file>;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        LineViewRenderObject {
            id: id_maker.next_id(),
            size: graphics::Vector2f::new(0.0, 0.0),
            _private: (),
            contents: self.contents,
            highlight_start: self.highlight_start,
            highlight_end: self.highlight_end,
        }
    }

    fn update_render_object(self, render_object: &mut Self::Result, _: &mut RenderObjectIdMaker) {
        // TODO: animate this
        render_object.contents = self.contents;
        render_object.highlight_start = self.highlight_start;
        render_object.highlight_end = self.highlight_end;
    }
}

impl<'file, Data> RenderObject<Data> for LineViewRenderObject<'file> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        let text = graphics::Text::new(self.contents, &graphics_context.font, 15); // TODO: control font size
        self.size = sc.clamp_size(text.global_bounds().size());
    }

    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, _: Option<RenderObjectId>) {
        // TODO: deal with overflow (clipping does not work because the bounding box does not include descenders)
        // util::clip(graphics_context, target, graphics::FloatRect::from_vecs(top_left, self.size), |target, top_left| {
        let mut text = graphics::Text::new(self.contents, &graphics_context.font, 15); // TODO: control font size
        text.set_position(top_left);
        text.set_fill_color(graphics::Color::WHITE); // TODO: control text color

        let highlight = match (self.highlight_start, self.highlight_end) {
            (None, None) => None,
            (None, Some(end)) => Some((0, end)),
            (Some(start), None) => Some((start, self.contents.len())),
            (Some(start), Some(end)) => Some((start, end)),
        };
        if let Some((start, end)) = highlight {
            let highlight_start_pos = text.find_character_pos(start);
            let highlight_end_pos = text.find_character_pos(end);

            let mut highlight_rect = graphics::RectangleShape::from_rect(graphics::FloatRect::from_vecs(highlight_start_pos, graphics::Vector2f::new(highlight_end_pos.x - highlight_start_pos.x, self.size.y)));

            highlight_rect.set_fill_color(graphics::Color::GREEN);

            target.draw(&highlight_rect);
        }

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
