pub(crate) mod label;
pub(crate) mod test_rect;

use crate::visualizer::render_object::{RenderObject, RenderObjectIdMaker};

pub(crate) trait Widget<Data: ?Sized> {
    type Result: RenderObject<Data>;
    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result;
    fn update_render_object(self, render_object: &mut Self::Result, id_maker: &mut RenderObjectIdMaker);
}
