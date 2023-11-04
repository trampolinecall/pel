use std::{collections::HashMap, hash::Hash};

use sfml::graphics::{Shape, Transformable};

use crate::{
    source::Span,
    visualizer::{
        event, graphics, layout,
        render_object::{
            animated::{Animated, AnimatedValue, Lerpable},
            RenderObject, RenderObjectId, RenderObjectIdMaker,
        },
        widgets::{center::Center, expand::Expand, fixed_size::fixed_size, flex, label::Label, min_size::MinSize, Widget},
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
    highlights: Vec<LineHighlight>,
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
#[derive(Copy, Clone, Eq, PartialEq)]
enum HighlightShownAmount {
    CompressedToLeft,
    AllShown,
    CompressedToRight,
}
pub(crate) struct LineViewRenderObject<'file, GetFont: Fn(&graphics::Fonts) -> &graphics::Font> {
    id: RenderObjectId,
    contents: &'file str,
    highlights: HashMap<LineHighlight, Animated<HighlightShownAmount>>,
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
pub(crate) fn code_view<'file, CodeFont: Fn(&graphics::Fonts) -> &graphics::Font + Copy + 'file, LineNrFont: Fn(&graphics::Fonts) -> &graphics::Font + Copy + 'file, Data: 'file>(
    primary_span: Span<'file>,
    secondary_highlights: impl IntoIterator<Item = (Span<'file>, graphics::Color)>,
    line_nr_font: LineNrFont,
    line_nr_font_size: u32,
    code_font: CodeFont,
    code_font_size: u32,
) -> impl Widget<Data> + 'file {
    let secondary_highlights: Vec<_> = secondary_highlights.into_iter().collect();
    Expand::new(flex::homogeneous::Flex::new(
        flex::Direction::Vertical,
        primary_span
            .file
            .lines
            .iter()
            .enumerate()
            .map(|(line_number, (line_bounds, line_contents))| {
                let highlights_on_line = std::iter::once(&(primary_span, graphics::Color::rgb(50, 100, 50)))
                    .chain(secondary_highlights.iter())
                    .flat_map(|(span, color)| {
                        let highlight_span_overlaps_line_bounds = !(span.end < line_bounds.start || span.start >= line_bounds.end);
                        if highlight_span_overlaps_line_bounds {
                            let highlight_start = if span.start < line_bounds.start { HighlightStartPosition::Start } else { HighlightStartPosition::Index(span.start - line_bounds.start) };
                            let highlight_end = if span.end > line_bounds.end { HighlightEndPosition::End } else { HighlightEndPosition::Index(span.end - line_bounds.start) };
                            Some(LineHighlight { start: highlight_start, end: highlight_end, color: *color })
                        } else {
                            None
                        }
                    })
                    .collect();
                (
                    flex::ItemSettings::Fixed,
                    flex! {
                        horizontal
                        line_number: flex::ItemSettings::Fixed, fixed_size(Center::new(Label::new((line_number + 1).to_string(), line_nr_font, line_nr_font_size)), graphics::Vector2f::new(20.0, 20.0)), // TODO: also don't hardcode this size, also TODO: line numbers should really be right aligned, not centered
                        line_view: flex::ItemSettings::Flex(1.0), MinSize::new(
                            LineView { contents: line_contents, highlights: highlights_on_line, get_font: code_font, font_size: code_font_size },
                            graphics::Vector2f::new(0.0, 20.0), // TODO: don't hardcode minimum height
                        ),
                    },
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
                .highlights
                .into_iter()
                .map(|r| {
                    let mut a = Animated::new(HighlightShownAmount::CompressedToLeft);
                    a.set(HighlightShownAmount::AllShown);
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
            .highlights
            .into_iter()
            .map(|requested| {
                let shown_amount = render_object.highlights.remove(&requested);
                match shown_amount {
                    Some(mut a) => {
                        match a.get() {
                            AnimatedValue::Steady(HighlightShownAmount::AllShown) | AnimatedValue::Animating { before: _, after: HighlightShownAmount::AllShown, amount: _ } => (requested, a), // the highlight is either in the middle of animating in or is completely shown, so allow it to continue unchanged
                            AnimatedValue::Steady(HighlightShownAmount::CompressedToRight | HighlightShownAmount::CompressedToLeft)
                            | AnimatedValue::Animating { before: _, after: HighlightShownAmount::CompressedToRight | HighlightShownAmount::CompressedToLeft, amount: _ } => {
                                // it really shouldnt be trying to animate towards being compressed to the left but there isnt a way to prove to the type system that so we must do this
                                // the highlight is either animating out or not shown, so make it animate in
                                a.set(HighlightShownAmount::AllShown);
                                (requested, a)
                            }
                        }
                    }
                    None => {
                        // if a highlight should be shown here but doesnt have an entry in the render object, create one
                        let mut a = Animated::new(HighlightShownAmount::CompressedToLeft);
                        a.set(HighlightShownAmount::AllShown);
                        (requested, a)
                    }
                }
            })
            .collect(); // TODO: is it possible to do this without collect?

        // the ones left over (i.e. that are not in the widget / requested by the widget) should be removed
        let left_over = render_object.highlights.drain().filter_map(|(highlight, mut shown)| match shown.get() {
            AnimatedValue::Steady(HighlightShownAmount::AllShown) | AnimatedValue::Animating { before: _, after: HighlightShownAmount::AllShown, amount: _ } => {
                // if it is either completely shown or animating in, make it animate out
                shown.set(HighlightShownAmount::CompressedToRight);
                Some((highlight, shown))
            }
            AnimatedValue::Steady(HighlightShownAmount::CompressedToLeft | HighlightShownAmount::CompressedToRight) => None, // if it is completely hidden, just remove it from the hashmap entirely
            AnimatedValue::Animating { before: _, after: HighlightShownAmount::CompressedToLeft | HighlightShownAmount::CompressedToRight, amount: _ } => Some((highlight, shown)), // if it is in the middle of animating out, allow to continue
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
            let highlight_start_index = match start {
                HighlightStartPosition::Start => 0,
                HighlightStartPosition::Index(i) => *i,
            };
            let highlight_end_index = match end {
                HighlightEndPosition::End => self.contents.len(),
                HighlightEndPosition::Index(i) => *i,
            };

            let (left_x_interp, width_amount) = {
                let get_positions_from_shown_amount = |shown_amount| match shown_amount {
                    HighlightShownAmount::CompressedToLeft => (0.0, 0.0),
                    HighlightShownAmount::AllShown => (0.0, 1.0),
                    HighlightShownAmount::CompressedToRight => (1.0, 0.0),
                };
                match shown_amount.get() {
                    AnimatedValue::Steady(sa) => get_positions_from_shown_amount(*sa),
                    AnimatedValue::Animating { before: start_sa, after: end_sa, amount: lerp_interp_amount } => {
                        let (start_left_x, start_width) = get_positions_from_shown_amount(*start_sa);
                        let (end_left_x, end_width) = get_positions_from_shown_amount(*end_sa);
                        (start_left_x.lerp(&end_left_x, lerp_interp_amount), start_width.lerp(&end_width, lerp_interp_amount))
                    }
                }
            };

            let highlight_start_pos = text.find_character_pos(highlight_start_index);
            let highlight_end_pos = text.find_character_pos(highlight_end_index);
            let highlight_width = highlight_end_pos.x - highlight_start_pos.x;

            let mut highlight_rect = graphics::RectangleShape::from_rect(graphics::FloatRect::from_vecs(
                highlight_start_pos + graphics::Vector2f::new(left_x_interp * highlight_width, 0.0),
                graphics::Vector2f::new(highlight_width * width_amount, self.size.y),
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

    fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::TargetedEvent) {}
    fn general_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::GeneralEvent) {}
}
