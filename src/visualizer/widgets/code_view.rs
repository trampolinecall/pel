use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use sfml::graphics::{Shape, Transformable};

use crate::{
    source::Span,
    visualizer::{
        event, graphics, layout,
        render_object::{
            animated::{Animated, Lerpable},
            RenderObject, RenderObjectId, RenderObjectIdMaker,
        },
        widgets::{expand::Expand, flex, min_size::MinSize, Widget},
    },
};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum HighlightStartPosition {
    Start,
    Index(usize),
}
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum HighlightEndPosition {
    End,
    Index(usize),
}

pub(crate) struct LineView<'file, GetFont: Fn(&graphics::Fonts) -> &graphics::Font> {
    contents: &'file str,
    highlight: Option<LineHighlight>, // TODO: multiple highlights?
    get_font: GetFont,
    font_size: u32,
}
// TODO: padding between lines

#[derive(PartialEq, Eq)]
struct LineHighlight {
    start: HighlightStartPosition,
    end: HighlightEndPosition,
    color: graphics::Color,
}
impl Hash for LineHighlight {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
        self.color.r.hash(state);
        self.color.g.hash(state);
        self.color.b.hash(state);
        self.color.a.hash(state);
    }
}

pub(crate) struct LineViewRenderObject<'file, GetFont: Fn(&graphics::Fonts) -> &graphics::Font> {
    id: RenderObjectId,
    contents: &'file str,
    highlights: HashMap<LineHighlight, Animated<bool>>,
    size: graphics::Vector2f,
    get_font: GetFont,
    font_size: u32,
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
pub(crate) fn code_view<'file, GetFont: Fn(&graphics::Fonts) -> &graphics::Font + Copy + 'file, Data: 'file>(span: Span<'file>, font: GetFont, font_size: u32) -> impl Widget<Data> + 'file {
    Expand::new(flex::homogeneous::Flex::new(
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
                        Some(LineHighlight { start: highlight_start, end: highlight_end, color: graphics::Color::rgb(50, 100, 50) })
                    } else {
                        None
                    }
                };
                (
                    flex::ItemSettings::Fixed,
                    MinSize::new(
                        LineView { contents: line_contents, highlight, get_font: font, font_size },
                        graphics::Vector2f::new(0.0, 20.0), // TODO: don't hardcode minimum height
                    ),
                )
            })
            .collect(),
    ))
}

impl<'file, GetFont: Fn(&graphics::Fonts) -> &graphics::Font, Data> Widget<Data> for LineView<'file, GetFont> {
    type Result = LineViewRenderObject<'file, GetFont>;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        LineViewRenderObject {
            id: id_maker.next_id(),
            contents: self.contents,
            highlights: self
                .highlight
                .into_iter()
                .map(|r| {
                    let mut a = Animated::new(false);
                    a.update(true);
                    (r, a)
                })
                .collect(),
            size: graphics::Vector2f::new(0.0, 0.0),
            get_font: self.get_font,
            font_size: self.font_size,
            _private: (),
        }
    }

    fn update_render_object(self, render_object: &mut Self::Result, _: &mut RenderObjectIdMaker) {
        render_object.contents = self.contents;

        // the requested highlights are the ones in the widget that should be kept or made
        let requested_highlights: Vec<_> = self
            .highlight
            .into_iter()
            .map(|requested| {
                let shown_amount = render_object.highlights.remove(&requested);
                match shown_amount {
                    Some(mut a) => {
                        match a.get() {
                            Ok(true) | Err((_, true, _)) => (requested, a), // the highlight is either in the middle of animating in or is completely shown, so allow it to continue unchanged
                            Ok(false) | Err((_, false, _)) => {
                                // the highlight is either animating out or not shown, so make it animate in
                                a.update(true);
                                (requested, a)
                            }
                        }
                    }
                    None => {
                        // if a highlight should be shown here but doesnt have an entry in the render object, create one
                        let mut a = Animated::new(false);
                        a.update(true);
                        (requested, a)
                    }
                }
            })
            .collect(); // TODO: is it possible to do this without collect?

        // the ones left over (i.e. that are not in the widget / requested by the widget) should be removed
        let left_over = render_object.highlights.drain().filter_map(|(highlight, mut shown)| match shown.get() {
            Ok(true) | Err((_, true, _)) => {
                // if it is either completely shown or animating in, make it animate out
                shown.update(false);
                Some((highlight, shown))
            }
            Ok(false) => None,                              // if it is completely hidden, just remove it from the hashmap entirely
            Err((_, false, _)) => Some((highlight, shown)), // if it is in the middle of animating out, allow to continue
        });

        render_object.highlights = requested_highlights.into_iter().chain(left_over).collect();
    }
}

impl<'file, GetFont: Fn(&graphics::Fonts) -> &graphics::Font, Data> RenderObject<Data> for LineViewRenderObject<'file, GetFont> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        let text = graphics::Text::new(self.contents, (self.get_font)(&graphics_context.fonts), self.font_size);
        self.size = sc.clamp_size(text.global_bounds().size());
    }

    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, _: Option<RenderObjectId>) {
        // TODO: deal with overflow (clipping does not work because the bounding box does not include descenders)
        // util::clip(graphics_context, target, graphics::FloatRect::from_vecs(top_left, self.size), |target, top_left| {
        let mut text = graphics::Text::new(self.contents, (self.get_font)(&graphics_context.fonts), self.font_size);
        text.set_position(top_left);
        text.set_fill_color(graphics::Color::WHITE); // TODO: control text color

        for (LineHighlight { start, end, color }, shown_amount) in &self.highlights {
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

            let shown_interp = match shown_amount.get() {
                Ok(true) => 1.0,
                Ok(false) => 0.0,
                Err((start, end, amount)) => (*start as i32 as f32).lerp(&(*end as i32 as f32), amount),
            };

            let mut highlight_rect = graphics::RectangleShape::from_rect(graphics::FloatRect::from_vecs(
                highlight_start_pos,
                graphics::Vector2f::new((highlight_end_pos.x - highlight_start_pos.x) * shown_interp, self.size.y),
            ));

            highlight_rect.set_fill_color(*color);

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
