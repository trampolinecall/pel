pub(crate) mod dom;
pub(crate) mod graphics;
pub(crate) mod lens;
pub(crate) mod render_object;
pub(crate) mod vdom;

#[macro_use]
pub(crate) mod widgets;

use std::{cell::RefCell, mem::ManuallyDrop, rc::Rc};

use wasm_bindgen::JsValue;

use crate::app::widgets::Widget;

struct App<Data, DataAsWidget: Widget<Data>, ToWidget: Fn(&Data) -> DataAsWidget> {
    data: Rc<RefCell<Data>>,
    dom: Rc<RefCell<dom::Dom>>,
    to_widget: ToWidget,
}

impl<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Fn(&Data) -> DataAsWidget + Copy + 'static> App<Data, DataAsWidget, ToWidget> {
    fn run(data: Data, to_widget: ToWidget) {
        let data = Rc::new(RefCell::new(data));

        let document = web_sys::window().expect("no global window").document().expect("no document on window");
        let app_div = document.get_element_by_id("app").expect("no div with id 'app'");

        let dom = Rc::new(RefCell::new(dom::Dom::new_empty(document, &app_div)));

        let app = Rc::new(RefCell::new(App { data, dom, to_widget }));

        App::update_dom(&app);
    }

    fn run_update(app_refcell: &Rc<RefCell<Self>>, event: JsValue, closure: &dyn Fn(JsValue, &mut Data)) {
        closure(event, &mut app_refcell.borrow().data.borrow_mut());
        App::update_dom(app_refcell);
    }
    fn update_dom(app_refcell: &Rc<RefCell<Self>>) {
        let app = app_refcell.borrow();
        let vdom = dom::vdom_with_closures::Element::from_normal_vdom(
            {
                let app_refcell = Rc::clone(app_refcell);
                move |event, update_closure| App::run_update(&app_refcell, event, update_closure)
            },
            &app.dom,
            &app.data,
            (app.to_widget)(&app.data.borrow()).to_vdom(),
            app.to_widget,
        );
        app.dom.borrow_mut().update(&app.dom, &app.data, vdom, app.to_widget);
    }
}

pub(crate) fn run<Data: 'static, DataAsWidget: Widget<Data> + 'static>(data: Data, to_widget: impl Fn(&Data) -> DataAsWidget + Copy + 'static) {
    App::run(data, to_widget);
}
