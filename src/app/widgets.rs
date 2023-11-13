pub(crate) mod center;
pub(crate) mod clickable;
pub(crate) mod either;
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

use crate::app::vdom::Element;

pub(crate) trait Widget<Data: ?Sized> {
    fn to_vdom(self) -> Element<Data>;
}
