pub(crate) mod graphics;
pub(crate) mod layout;
pub(crate) mod lens;
pub(crate) mod render_object;
#[macro_use]
pub(crate) mod widgets;
pub(crate) mod dom;

use std::mem::ManuallyDrop;

use crate::visualizer::widgets::Widget;

pub(crate) fn run<Model: 'static, ModelAsWidget: Widget<Model>>(model: Model, model_to_widget: impl Fn(&Model) -> ModelAsWidget) {
    // use ManuallyDrop to prevent the model and actual dom (which contains callbacks that the dom references) from being dropped when this function returns
    // that means that the model and actual dom will be leaked, but those are supposed to stay alive for the lifetime for as long as the program runs, so this is actually intended behavior

    let model = ManuallyDrop::new(model);

    let document = web_sys::window().expect("no global window").document().expect("no document on window");
    let app_div = document.get_element_by_id("app").expect("no div with id 'app'");

    let mut actual_dom = ManuallyDrop::new(dom::ActualDom::new(&document, model_to_widget(&model).to_vdom()));
    app_div.append_child(actual_dom.dom()).unwrap(); // TODO: handle this error properly

    // TODO: figure out how this is actually supposed to work because it is not supposed to be in an infinite "event loop" (present once, only present again when callbacks run?)
    // loop {
    let new_vdom = model_to_widget(&model).to_vdom();
    actual_dom.update(&document, new_vdom);
    // }
}
