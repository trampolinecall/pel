pub(crate) mod center;
pub(crate) mod clickable;
pub(crate) mod either;
pub(crate) mod empty;
pub(crate) mod expand;
#[macro_use]
pub(crate) mod flex;
pub(crate) mod code_view;
pub(crate) mod fixed_size;
pub(crate) mod label;
pub(crate) mod max_size;
pub(crate) mod min_size;
pub(crate) mod padding;
pub(crate) mod responds_to_keyboard;
pub(crate) mod test_rect;
pub(crate) mod vsplit;

use crate::visualizer::render_object::{RenderObject, RenderObjectIdMaker};

pub(crate) trait Widget<Data: ?Sized> {
    type Result: RenderObject<Data>;
    fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result;
    fn update_render_object(self, render_object: &mut Self::Result, id_maker: &mut RenderObjectIdMaker);
}
