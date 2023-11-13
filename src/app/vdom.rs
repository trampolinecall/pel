use std::collections::HashMap;

use wasm_bindgen::JsValue;

// TODO: it seems a bit strange to have the Data parameter here; figure out a better way?
pub(crate) struct Element<Data: ?Sized> {
    pub(crate) type_: ElementType,
    pub(crate) props: HashMap<String, JsValue>,                                       // TODO: make this &'static str instead of String?
    pub(crate) event_listeners: Vec<(&'static str, Box<dyn Fn(JsValue, &mut Data)>)>, // TODO: do not box closures?
    pub(crate) children: Vec<Node<Data>>,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum ElementType {
    Div,
    P,
    Pre,
    Code,
}
pub(crate) enum Node<Data: ?Sized> {
    Element(Element<Data>),
    Text(String),
}

impl ElementType {
    pub(crate) fn stringify(&self) -> &'static str {
        match self {
            ElementType::Div => "div",
            ElementType::P => "p",
            ElementType::Pre => "pre",
            ElementType::Code => "code",
        }
    }
}
