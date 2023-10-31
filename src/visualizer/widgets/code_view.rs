use sfml::graphics::{Shape, Transformable};

use crate::visualizer::render_object::animated::{Animated, Lerpable};
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

#[derive(Copy, Clone, Eq, PartialEq)]
enum HighlightStartPosition {
    Start,
    Index(usize),
}
#[derive(Copy, Clone, Eq, PartialEq)]
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
    highlight: Animated<Option<(HighlightStartPosition, HighlightEndPosition)>>, // TODO: improve animation
    size: graphics::Vector2f,
    _private: (),
}

impl Lerpable for Option<(HighlightStartPosition, HighlightEndPosition)> {
    fn lerp(&self, other: &Self, amount: f64) -> Self {
        let (start_positions, end_positions) = match (self, other) {
            (None, None) => return None,
            (None, Some(end_positions)) => ((HighlightStartPosition::Start, HighlightEndPosition::Index(0)), *end_positions),
            (Some(start_positions), None) => (*start_positions, (HighlightStartPosition::Start, HighlightEndPosition::Index(0))), // TODO: figure this out better
            (Some(start_positions), Some(end_positions)) => (*start_positions, *end_positions),
        };

        let highlight_start_lerped = start_positions.0.lerp(&end_positions.0, amount);
        let highlight_end_lerped = start_positions.1.lerp(&end_positions.1, amount);

        Some((highlight_start_lerped, highlight_end_lerped))
    }
}
impl Lerpable for HighlightStartPosition {
    fn lerp(&self, other: &Self, amount: f64) -> Self {
        // TODO: figure out how to deal with indexes because they cannot be floats
        let convert_start_to_0 = |position: &_| match position {
            HighlightStartPosition::Start => 0,
            HighlightStartPosition::Index(i) => *i,
        };
        let start = convert_start_to_0(self);
        let end = convert_start_to_0(other);
        HighlightStartPosition::Index(start.lerp(&end, amount))
    }
}
impl Lerpable for HighlightEndPosition {
    fn lerp(&self, other: &Self, amount: f64) -> Self {
        // TODO: figure out how to deal with indexes because they cannot be floats
        let convert_end_to_100 = |position: &_| match position {
            HighlightEndPosition::End => 100, // TODO: do this better
            HighlightEndPosition::Index(i) => *i,
        };
        let start = convert_end_to_100(self);
        let end = convert_end_to_100(other);
        HighlightEndPosition::Index(start.lerp(&end, amount))
    }
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
        LineViewRenderObject { id: id_maker.next_id(), contents: self.contents, highlight: Animated::new(self.highlight), size: graphics::Vector2f::new(0.0, 0.0), _private: () }
    }

    fn update_render_object(self, render_object: &mut Self::Result, _: &mut RenderObjectIdMaker) {
        render_object.contents = self.contents;
        render_object.highlight.update(self.highlight);
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

        if let Some((start, end)) = &self.highlight.get_lerped() {
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
