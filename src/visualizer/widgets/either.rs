use std::marker::PhantomData;

use crate::visualizer::{dom, widgets::Widget};

pub(crate) enum Either<Data, Left: Widget<Data>, Right: Widget<Data>> {
    Left(Left),
    Right(Right, PhantomData<fn(&mut Data)>),
}

/* TODO: REMOVE
pub(crate) enum EitherRenderObject<Data, Left: RenderObject<Data>, Right: RenderObject<Data>> {
    Left(Left),
    Right(Right, PhantomData<fn(&mut Data)>),
}
*/

impl<Data, Left: Widget<Data>, Right: Widget<Data>> Either<Data, Left, Right> {
    pub(crate) fn new_left(left: Left) -> Self {
        Self::Left(left)
    }
    pub(crate) fn new_right(right: Right) -> Self {
        Self::Right(right, PhantomData)
    }
}

impl<Data, Left: Widget<Data>, Right: Widget<Data>> Widget<Data> for Either<Data, Left, Right> {
    fn to_vdom(self) -> dom::Element<Data> {
        match self {
            Either::Left(l) => l.to_vdom(),
            Either::Right(r, _) => r.to_vdom(),
        }
    }
}
/* TODO: REMOVE
impl<Data, Left: RenderObject<Data>, Right: RenderObject<Data>> RenderObject<Data> for EitherRenderObject<Data, Left, Right> {
    fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
        match self {
            EitherRenderObject::Left(l) => l.layout(graphics_context, sc),
            EitherRenderObject::Right(r, _) => r.layout(graphics_context, sc),
        }
    }

    fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>) {
        match self {
            EitherRenderObject::Left(l) => l.draw(graphics_context, target, top_left, hover),
            EitherRenderObject::Right(r, _) => r.draw(graphics_context, target, top_left, hover),
        }
    }

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId> {
        match self {
            EitherRenderObject::Left(l) => l.find_hover(top_left, mouse),
            EitherRenderObject::Right(r, _) => r.find_hover(top_left, mouse),
        }
    }

    fn size(&self) -> graphics::Vector2f {
        match self {
            EitherRenderObject::Left(l) => l.size(),
            EitherRenderObject::Right(r, _) => r.size(),
        }
    }

    fn send_targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, target: RenderObjectId, event: TargetedEvent) {
        match self {
            EitherRenderObject::Left(l) => l.send_targeted_event(top_left, data, target, event),
            EitherRenderObject::Right(r, _) => r.send_targeted_event(top_left, data, target, event),
        }
    }

    fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: GeneralEvent) {}
}
*/
