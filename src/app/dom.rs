use std::collections::HashMap;

use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::js_sys;

use crate::app::vdom;

pub(crate) struct Dom {
    document: web_sys::Document,
    root: Element,
}

// includes vdom parts in order to diff with the next vdom
struct Element {
    dom_element: web_sys::Element,
    type_: vdom::ElementType,
    props: HashMap<&'static str, JsValue>, // TODO: make this &'static str instead of String?
    event_listeners: Vec<(&'static str, Closure<dyn Fn(JsValue)>)>,
    children: Vec<Node>,
}
enum Node {
    Element(Element),
    Text(web_sys::Text, String),
}

impl Dom {
    pub(crate) fn new_empty(document: web_sys::Document, parent: &web_sys::Node) -> Dom {
        let dom_element = document.create_element(vdom::ElementType::Div.stringify()).unwrap(); // TODO: handle errors properly
        parent.append_child(&dom_element).unwrap(); // TODO: handle this error properly
        let elem = Element { dom_element, type_: vdom::ElementType::Div, props: HashMap::new(), event_listeners: Vec::new(), children: Vec::new() };
        Dom { document, root: elem }
    }

    pub(crate) fn update<Data: 'static>(&mut self, new_vdom: vdom::Element<Data>, run_update: &(impl Fn(JsValue, &dyn Fn(JsValue, &mut Data)) + Clone + 'static)) {
        self.root.update(&self.document, run_update, new_vdom);
    }
}
impl Element {
    fn new_from_vdom<Data: 'static>(document: &web_sys::Document, run_update: &(impl Fn(JsValue, &dyn Fn(JsValue, &mut Data)) + Clone + 'static), vdom: vdom::Element<Data>) -> Element {
        // TODO: handle all the errors properly (all the unwraps)
        let dom_element = document.create_element(vdom.type_.stringify()).unwrap();

        for (prop_name, prop_value) in &vdom.props {
            js_sys::Reflect::set(&dom_element, &(*prop_name).into(), prop_value).unwrap();
        }

        let event_listeners = Self::convert_event_listeners_from_vdom(run_update, vdom.event_listeners);
        Self::add_event_listeners(&dom_element, &event_listeners);

        let children = vdom
            .children
            .into_iter()
            .map(|child| {
                let run_update = run_update.clone();
                let node = Node::new_from_vdom(document, &run_update, child);
                dom_element.append_child(node.as_node()).unwrap(); // TODO: handle unwrap correctly
                node
            })
            .collect();

        Element { dom_element, type_: vdom.type_, props: vdom.props, event_listeners, children }
    }

    fn update<Data: 'static>(&mut self, document: &web_sys::Document, run_update: &(impl Fn(JsValue, &dyn Fn(JsValue, &mut Data)) + Clone + 'static), new_vdom: vdom::Element<Data>) {
        if self.type_ != new_vdom.type_ {
            // TODO: handle error properly
            let new_elem = Self::new_from_vdom(document, run_update, new_vdom);
            self.dom_element.replace_with_with_node_1(&new_elem.dom_element).unwrap();
            *self = new_elem;
        } else {
            // update properties
            // remove old properties
            self.props.retain(|old_prop_name, _| {
                if !new_vdom.props.contains_key(old_prop_name) {
                    // TODO: deal with error properly
                    js_sys::Reflect::delete_property(&self.dom_element, &(*old_prop_name).into()).unwrap();
                    false
                } else {
                    true
                }
            });

            // set new properties
            for (new_prop_name, new_value) in new_vdom.props {
                // TODO: deal with error properly
                // TODO: only set if value changed?
                js_sys::Reflect::set(&self.dom_element, &(*new_prop_name).into(), &new_value).unwrap();
                self.props.insert(new_prop_name, new_value);
            }

            Self::remove_event_listeners(&self.dom_element, &self.event_listeners);
            self.event_listeners.clear();
            self.event_listeners = Self::convert_event_listeners_from_vdom(run_update, new_vdom.event_listeners);
            Self::add_event_listeners(&self.dom_element, &self.event_listeners);

            // update children
            {
                let child_doms = self.dom_element.child_nodes();
                assert_eq!(child_doms.length() as usize, self.children.len());
                // remove children if there are too many
                if self.children.len() > new_vdom.children.len() {
                    for remove_i in (new_vdom.children.len()..self.children.len()).rev() {
                        web_sys::console::log_3(&remove_i.into(), &self.children.len().into(), &child_doms.length().into());
                        let this_node = child_doms.get(remove_i as u32).expect("index should be in range of child list");
                        let parent_node = this_node.parent_node().expect("node should have parent");
                        parent_node.remove_child(&this_node).unwrap(); // TODO: handle error properly
                        self.children.remove(remove_i);
                    }
                }
                let mut new_children = new_vdom.children.into_iter();
                // update the children
                for child in self.children.iter_mut() {
                    let new_vdom = new_children.next().expect("there should not be more children in the properties at this point because they have been removed in the if statement above");
                    child.update(document, run_update, new_vdom);
                }
                // add the new children if there are any more
                for vdom_to_add in new_children {
                    let node = Node::new_from_vdom(document, run_update, vdom_to_add);
                    self.dom_element.append_child(node.as_node()).unwrap(); // TODO: handle unwrap correctly
                    self.children.push(node);
                }
            }
        }
    }
    fn add_event_listeners(node: &web_sys::Node, event_listeners: &[(&str, Closure<dyn Fn(JsValue)>)]) {
        for (event, listener) in event_listeners {
            // TODO: handle error correctly instead of unwrapping
            node.add_event_listener_with_callback(event, listener.as_ref().unchecked_ref()).unwrap();
        }
    }

    fn remove_event_listeners(dom: &web_sys::Element, event_listeners: &[(&str, Closure<dyn Fn(JsValue)>)]) {
        for (event, listener) in event_listeners {
            // TODO: handle error correctly instead of unwrapping
            dom.remove_event_listener_with_callback(event, listener.as_ref().unchecked_ref()).unwrap();
        }
    }

    fn convert_event_listeners_from_vdom<Data: 'static>(
        run_update: &(impl Fn(JsValue, &dyn Fn(JsValue, &mut Data)) + Clone + 'static),
        event_listeners: Vec<(&'static str, Box<dyn Fn(JsValue, &mut Data)>)>,
    ) -> Vec<(&'static str, Closure<dyn Fn(JsValue)>)> {
        event_listeners
            .into_iter()
            .map(|(event, event_listener)| {
                (
                    event,
                    Closure::<dyn Fn(JsValue)>::new({
                        let run_update = run_update.clone();
                        move |event| run_update(event, &event_listener)
                    }),
                )
            })
            .collect()
    }
}
impl Node {
    fn new_from_vdom<Data: 'static>(document: &web_sys::Document, run_update: &(impl Fn(JsValue, &dyn Fn(JsValue, &mut Data)) + Clone + 'static), vdom: vdom::Node<Data>) -> Node {
        match vdom {
            vdom::Node::Element(e) => Node::Element(Element::new_from_vdom(document, run_update, e)),
            vdom::Node::Text(t) => Node::Text(document.create_text_node(&t), t),
        }
    }
    fn update<Data: 'static>(&mut self, document: &web_sys::Document, run_update: &(impl Fn(JsValue, &dyn Fn(JsValue, &mut Data)) + Clone + 'static), new_vdom: vdom::Node<Data>) {
        match (self, new_vdom) {
            (Node::Element(old_elem), vdom::Node::Element(new_vdom)) => {
                old_elem.update(document, run_update, new_vdom);
            }
            (Node::Text(text_node, old_text), vdom::Node::Text(new_text)) => {
                text_node.set_data(&new_text);
                *old_text = new_text;
            }
            (old_node, new_vdom) => {
                // TODO: handle error properly
                let parent_node = old_node.as_node().parent_node().expect("node should have parent");
                let new_node = Node::new_from_vdom(document, run_update, new_vdom);
                parent_node.replace_child(new_node.as_node(), old_node.as_node()).unwrap();
                *old_node = new_node;
            }
        }
    }

    fn as_node(&self) -> &web_sys::Node {
        match self {
            Node::Element(e) => &e.dom_element,
            Node::Text(n, _) => n,
        }
    }
}
