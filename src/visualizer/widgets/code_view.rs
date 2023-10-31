use sfml::graphics::{Shape, Transformable};

use crate::visualizer::widgets::flex;
use crate::visualizer::widgets::min_size::MinSize;
use crate::{
    source::Span,
    visualizer::{
        event, graphics, layout,
        render_object::{RenderObject, RenderObjectId, RenderObjectIdMaker},
        widgets::Widget,
    },
};

enum HighlightStartPosition {
    Start,
    Index(usize),
}
enum HighlightEndPosition {
    End,
    Index(usize),
}

pub(crate) struct LineView<'file> {
    contents: &'file str,
    highlight: Option<(HighlightStartPosition, HighlightEndPosition)>,
}
// TODO: padding between lines

pub(crate) struct LineViewRenderObject<'file> {
    id: RenderObjectId,
    contents: &'file str,
    highlight: Option<(HighlightStartPosition, HighlightEndPosition)>,
    size: graphics::Vector2f,
    _private: (),
}

// TODO: secondary spans with other messages, ...
// TODO: scrolling
pub(crate) fn code_view<'file, Data: 'file>(span: Span<'file>) -> impl Widget<Data> + 'file {
    flex::homogeneous::Flow::new(
        flex::Direction::Vertical,
        span.file
            .lines
            .iter()
            .map(|(line_bounds, line_contents)| {
                let highlight = {
                    let highlight_span_overlaps_line_bounds = !(span.end < line_bounds.start || span.start >= line_bounds.end);
                    if highlight_span_overlaps_line_bounds {
                        let highlight_start = if span.start < line_bounds.start { HighlightStartPosition::Start } else { HighlightStartPosition::Index(span.start - line_bounds.start) };
                        let highlight_end = if span.end > line_bounds.end { HighlightEndPosition::End } else { HighlightEndPosition::Index(span.end - line_bounds.start) };
                        Some((highlight_start, highlight_end))
                    } else {
                        None
                    }
                };
                (
                    flex::ItemSettings::Fixed,
                    MinSize::new(
                        LineView { contents: line_contents, highlight },
                        graphics::Vector2f::new(0.0, 20.0), // TODO: don't hardcode minimum height
                    ),
                )
            })
            .collect(),
    )
}

impl<'file, Data> Widget<Data> for LineView<'file> {
    type Result = LineViewRenderObject<'file>;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        LineViewRenderObject { id: id_maker.next_id(), contents: self.contents, highlight: self.highlight, size: graphics::Vector2f::new(0.0, 0.0), _private: () }
    }

    fn update_render_object(self, render_object: &mut Self::Result, _: &mut RenderObjectIdMaker) {
        // TODO: animate this
        render_object.contents = self.contents;
        render_object.highlight = self.highlight;
    }
}

impl<'file, Data> RenderObject<Data> for LineViewRenderObject<'file> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        let text = graphics::Text::new(self.contents, &graphics_context.monospace_font, 15); // TODO: control font size
        self.size = sc.clamp_size(text.global_bounds().size());
    }

    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, _: Option<RenderObjectId>) {
        // TODO: deal with overflow (clipping does not work because the bounding box does not include descenders)
        // util::clip(graphics_context, target, graphics::FloatRect::from_vecs(top_left, self.size), |target, top_left| {
        let mut text = graphics::Text::new(self.contents, &graphics_context.monospace_font, 15); // TODO: control font size
        text.set_position(top_left);
        text.set_fill_color(graphics::Color::WHITE); // TODO: control text color

        if let Some((start, end)) = &self.highlight {
            let highlight_start_pos = match start {
                HighlightStartPosition::Start => 0,
                HighlightStartPosition::Index(i) => *i,
            };
            let highlight_end_pos = match end {
                HighlightEndPosition::End => self.contents.len(),
                HighlightEndPosition::Index(i) => *i,
            };
            let highlight_start_pos = text.find_character_pos(highlight_start_pos);
            let highlight_end_pos = text.find_character_pos(highlight_end_pos);

            let mut highlight_rect =
                graphics::RectangleShape::from_rect(graphics::FloatRect::from_vecs(highlight_start_pos, graphics::Vector2f::new(highlight_end_pos.x - highlight_start_pos.x, self.size.y)));

            highlight_rect.set_fill_color(graphics::Color::rgb(50, 100, 50));

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
