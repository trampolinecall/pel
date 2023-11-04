use std::marker::PhantomData;

use crate::visualizer::{
    event::{GeneralEvent, TargetedEvent},
    graphics, layout,
    render_object::{RenderObject, RenderObjectId, RenderObjectIdMaker},
    widgets::Widget,
};

pub(crate) enum Either<Data, Left: Widget<Data>, Right: Widget<Data>> {
    Left(Left),
    Right(Right, PhantomData<fn(&mut Data)>),
}

pub(crate) enum EitherRenderObject<Data, Left: RenderObject<Data>, Right: RenderObject<Data>> {
    Left(Left),
    Right(Right, PhantomData<fn(&mut Data)>),
}

impl<Data, Left: Widget<Data>, Right: Widget<Data>> Either<Data, Left, Right> {
    pub(crate) fn new_left(left: Left) -> Self {
        Self::Left(left)
    }
    pub(crate) fn new_right(right: Right) -> Self {
        Self::Right(right, PhantomData)
    }
}

impl<Data, Left: Widget<Data>, Right: Widget<Data>> Widget<Data> for Either<Data, Left, Right> {
    type Result = EitherRenderObject<Data, <Left as Widget<Data>>::Result, <Right as Widget<Data>>::Result>;

    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
        match self {
            Either::Left(l) => EitherRenderObject::Left(l.to_render_object(id_maker)),
            Either::Right(r, phantom) => EitherRenderObject::Right(r.to_render_object(id_maker), phantom),
        }
    }

    fn update_render_object(self, render_object: &mut Self::Result, id_maker: &mut RenderObjectIdMaker) {
        match (self, render_object) {
            (Either::Left(widget), EitherRenderObject::Left(ro)) => widget.update_render_object(ro, id_maker),
            (Either::Right(widget, _), EitherRenderObject::Right(ro, _)) => widget.update_render_object(ro, id_maker),
            (self_, render_object) => *render_object = self_.to_render_object(id_maker),
        }
    }
}
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
