use std::marker::PhantomData;

use crate::visualizer::{
    vdom,
    widgets::{
        flex::{Direction, ItemSettings, _layout},
        Widget,
    },
};

pub(crate) struct Flex<Data, Child: Widget<Data>> {
    direction: Direction,
    children: Vec<(ItemSettings, Child)>,

    _phantom: PhantomData<fn(&mut Data)>,
}
/* TODO: REMOVE
pub(crate) struct FlexRenderObject<Data, Child: RenderObject<Data>> {
    direction: Direction,
    children: Vec<(Animated<ItemSettings>, graphics::Vector2f, Child)>,

    own_size: graphics::Vector2f,

    _phantom: PhantomData<fn(&mut Data)>,
    _private: (),
}
*/

impl<Data, Child: Widget<Data>> Flex<Data, Child> {
    pub(crate) fn new(direction: Direction, children: Vec<(ItemSettings, Child)>) -> Self {
        Self { direction, children, _phantom: PhantomData }
    }
    pub(crate) fn new_horizontal(children: Vec<(ItemSettings, Child)>) -> Self {
        Self::new(Direction::Horizontal, children)
    }
    pub(crate) fn new_vertical(children: Vec<(ItemSettings, Child)>) -> Self {
        Self::new(Direction::Vertical, children)
    }
}

impl<Data, Child: Widget<Data>> Widget<Data> for Flex<Data, Child> {
    fn to_vdom(self) -> vdom::Element<Data> {
        todo!()
    }
}
/* TODO: REMOVE
impl<Data, Child: RenderObject<Data>> RenderObject<Data> for FlexRenderObject<Data, Child> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        // lay out fixed elements and count up total flex scaling factors
        let mut total_flex_scale = 0.0;
        let mut major_size_left = self.direction.take_major_component(sc.max);
        for (settings, _, child) in &mut self.children {
            _layout::first_phase_step(graphics_context, sc, self.direction, &mut total_flex_scale, &mut major_size_left, _layout::animated_settings(*settings), child);
        }

        // lay out all of the flex children
        for (settings, _, child) in &mut self.children {
            _layout::second_phase_step(graphics_context, sc, self.direction, total_flex_scale, major_size_left, _layout::animated_settings(*settings), child);
        }

        // assign each of the offsets and calcaulte own_size
        let mut major_offset = 0.0;
        let mut max_minor_size = 0.0;
        for (_, offset, child) in &mut self.children {
            *offset = _layout::third_phase_step(self.direction, &mut major_offset, &mut max_minor_size, child);
        }
        self.own_size = sc.clamp_size(self.direction.make_vector_in_direction(major_offset, max_minor_size));
    }

    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>) {
        for (_, offset, child) in &self.children {
            child.draw(graphics_context, target, top_left + *offset, hover);
        }
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId> {
        for (_, offset, child) in &self.children {
            if let Some(hover) = child.find_hover(top_left + *offset, mouse) {
                return Some(hover);
            }
        }
        None
    }

    fn size(&self) -> graphics::Vector2f {
        self.own_size
    }

    fn send_targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, target: RenderObjectId, event: TargetedEvent) {
        for (_, offset, child) in &mut self.children {
            child.send_targeted_event(top_left + *offset, data, target, event);
        }
    }

    fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: GeneralEvent) {}
}
*/
