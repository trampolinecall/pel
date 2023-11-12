use std::collections::HashMap;

use wasm_bindgen::JsValue;

// TODO: it seems a bit strange to have the Data parameter here; figure out a better way?
pub(crate) struct Element<Data: ?Sized> {
    pub(crate) type_: ElementType,
    pub(crate) props: HashMap<String, JsValue>,
    pub(crate) event_listeners: Vec<(String, Box<dyn Fn(&mut Data)>)>,
    pub(crate) children: Vec<Element<Data>>,
}
pub(crate) enum ElementType {}

impl<Data> Element<Data> {
    pub(crate) fn update_actual_dom(&self, dom: web_sys::Element) {
        todo!()
    }
}
