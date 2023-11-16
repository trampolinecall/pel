pub(crate) mod dom;
pub(crate) mod graphics;
pub(crate) mod lens;
pub(crate) mod render_object;
pub(crate) mod vdom;

#[macro_use]
pub(crate) mod widgets;

use std::{cell::RefCell, rc::Rc};

use crate::app::widgets::Widget;

pub(crate) fn run<Model: 'static, ModelAsWidget: Widget<Model> + 'static>(model: Model, model_to_widget: impl Fn(&Model) -> ModelAsWidget + Copy + 'static) {
    let model = Rc::new(RefCell::new(model));

    let document = web_sys::window().expect("no global window").document().expect("no document on window");
    let app_div = document.get_element_by_id("app").expect("no div with id 'app'");

    let dom = dom::Dom::new(&model, document, &app_div, model_to_widget);
}
