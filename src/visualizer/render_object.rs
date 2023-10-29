use sfml::graphics::Transformable;

use crate::visualizer::{
    event::{GeneralEvent, TargetedEvent},
    graphics,
    layout::SizeConstraints,
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct RenderObjectId(u64);

pub(crate) struct RenderObjectIdMaker(u64);
impl RenderObjectIdMaker {
    pub(crate) fn new() -> RenderObjectIdMaker {
        RenderObjectIdMaker(0)
    }
    pub(crate) fn next_id(&mut self) -> RenderObjectId {
        let id = RenderObjectId(self.0);
        self.0 += 1;
        id
    }
}

pub(crate) trait RenderObject<Data: ?Sized> {
    // TODO: automate send_targeted_event and find_hover by having iter_children_by_z method?

    fn layout(&mut self, sc: SizeConstraints);
    // top_left in draw_inner and draw does not necessarily correspond to window coordinates, but top_left in find_hover always does
    fn draw(&self, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>) {
        let mut sub_graphics =
            graphics::RenderTexture::with_settings(self.size().x.ceil() as u32, self.size().y.ceil() as u32, &graphics::default_render_context_settings()).expect("could not create render texture");

        sub_graphics.set_active(true);
        self.draw_inner(&mut sub_graphics, graphics::Vector2f::new(0.0, 0.0), hover);
        sub_graphics.set_active(false);
        sub_graphics.display();

        let mut sprite = graphics::Sprite::new();
        sprite.set_texture(sub_graphics.texture(), true);
        sprite.set_position(top_left);
        target.draw(&sprite);
    }

    fn draw_inner(&self, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>);
    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId>;
    fn size(&self) -> graphics::Vector2f;

    fn send_targeted_event(&self, data: &mut Data, target: RenderObjectId, event: TargetedEvent);
    fn targeted_event(&self, data: &mut Data, event: TargetedEvent);
    fn general_event(&self, data: &mut Data, event: GeneralEvent);
}
