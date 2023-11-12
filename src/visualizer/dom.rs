use std::collections::HashMap;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Reflect;

// the dom and the vdom that created it
pub(crate) struct ActualDom<Data> {
    dom: web_sys::Element,
    vdom: Element<Data>,
}

// TODO: it seems a bit strange to have the Data parameter here; figure out a better way?
pub(crate) struct Element<Data: ?Sized> {
    pub(crate) type_: ElementType,
    pub(crate) props: HashMap<String, JsValue>, // TODO: make this &'static str instead of String?
    pub(crate) event_listeners: Vec<(&'static str, Box<dyn Fn(&mut Data)>)>,
    pub(crate) children: Vec<ElementChild<Data>>,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum ElementType {
    Div,
    P,
}
pub(crate) enum ElementChild<Data: ?Sized> {
    Element(Element<Data>),
    Text(String),
}

impl ElementType {
    fn stringify(&self) -> &'static str {
        match self {
            ElementType::Div => "div",
            ElementType::P => "p",
        }
    }
}
impl<Data> ActualDom<Data> {
    pub(crate) fn new(document: &web_sys::Document, vdom: Element<Data>) -> ActualDom<Data> {
        ActualDom { dom: make_dom(document, &vdom), vdom }
    }

    pub(crate) fn dom(&self) -> &web_sys::Element {
        &self.dom
    }
    pub(crate) fn update(&mut self, document: &web_sys::Document, new_vdom: Element<Data>) {
        if self.vdom.type_ != new_vdom.type_ {
            *self = ActualDom::new(document, new_vdom);
        } else {
            update_dom(document, &self.dom, &self.vdom, &new_vdom);
            self.vdom = new_vdom;
        }
    }
}

fn make_child<Data>(document: &web_sys::Document, child: &ElementChild<Data>, parent: &web_sys::Element) {
    // TODO: handle all the errors properly (all the unwraps) (maybe also grep over all code too?)
    match child {
        ElementChild::Element(e) => {
            parent.append_child(&make_dom(document, e)).unwrap();
        }
        ElementChild::Text(t) => {
            parent.append_child(&document.create_text_node(t)).unwrap();
        }
    }
}
fn make_dom<Data>(document: &web_sys::Document, vdom: &Element<Data>) -> web_sys::Element {
    // TODO: handle all the errors properly (all the unwraps)
    let elem = document.create_element(vdom.type_.stringify()).unwrap();

    for (prop_name, prop_value) in &vdom.props {
        Reflect::set(&elem, &prop_name.into(), prop_value).unwrap();
    }

    /* TODO:
    for (event, listener) in &vdom.event_listeners {
    }
    */

    for child in &vdom.children {
        make_child(document, child, &elem);
    }

    elem
}

fn update_child<Data>(document: &web_sys::Document, dom: web_sys::Node, old_vdom: &ElementChild<Data>, new_vdom: &ElementChild<Data>) {
    match (old_vdom, new_vdom) {
        (ElementChild::Element(old_vdom), ElementChild::Element(new_vdom)) => {
            update_dom(document, dom.dyn_ref().expect("if both old vdom and new vdom are elements, the current dom should be element"), old_vdom, new_vdom);
        }
        (ElementChild::Text(_), ElementChild::Text(new_text)) => {
            dom.set_node_value(Some(new_text));
        }
        (_, ElementChild::Text(new_text)) => {
            // TODO: handle error properly
            let parent_node = dom.parent_node().expect("node should have parent");
            parent_node.replace_child(&document.create_text_node(new_text), &dom).unwrap();
        }
        (_, ElementChild::Element(new_vdom)) => {
            // TODO: handle error properly
            let parent_node = dom.parent_node().expect("node should have parent");
            parent_node.replace_child(&make_dom(document, new_vdom), &dom).unwrap();
        }
    }
}
fn update_dom<Data>(document: &web_sys::Document, dom: &web_sys::Element, old_vdom: &Element<Data>, new_vdom: &Element<Data>) {
    // update properties
    // remove old properties
    for old_prop_name in old_vdom.props.keys() {
        if !new_vdom.props.contains_key(old_prop_name) {
            // TODO: deal with error properly
            Reflect::delete_property(dom, &old_prop_name.into()).unwrap();
        }
    }
    for (new_prop_name, new_value) in &new_vdom.props {
        // TODO: deal with error properly
        // TODO: only set if value changed?
        Reflect::set(dom, &new_prop_name.into(), new_value).unwrap();
    }

    // TODO: update event listeners

    // update children
    {
        let child_doms = dom.child_nodes();
        let update_child_doms = || {
            for (index, (old_vdom, new_vdom)) in old_vdom.children.iter().zip(&new_vdom.children).enumerate() {
                let dom = child_doms.get(index as u32).expect("index should be in range of child list");
                update_child(document, dom, old_vdom, new_vdom);
            }
        };
        assert_eq!(child_doms.length() as usize, old_vdom.children.len());
        match old_vdom.children.len().cmp(&new_vdom.children.len()) {
            std::cmp::Ordering::Less => {
                // children need to be added
                update_child_doms();
                for vdom_to_add in new_vdom.children.iter().skip(old_vdom.children.len()) {
                    make_child(document, vdom_to_add, dom);
                }
            }
            std::cmp::Ordering::Equal => {
                // same number of children
                update_child_doms();
            }
            std::cmp::Ordering::Greater => {
                // children need to be removed
                for remove_i in new_vdom.children.len()..old_vdom.children.len() {
                    let this_node = child_doms.get(remove_i as u32).expect("index should be in range of child list");
                    let parent_node = this_node.parent_node().expect("node should have parent");
                    parent_node.remove_child(&this_node).unwrap(); // TODO: handle error properly
                }
                update_child_doms();
            }
        }
    }
}
