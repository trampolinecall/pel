use std::{cell::RefCell, collections::HashMap, rc::Rc};

use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::js_sys;

use crate::app::{vdom, widgets::Widget};

// the vdom but with wasm Closures instead of rust Box<dyn Fn(...)>
mod vdom_with_closures {
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use wasm_bindgen::{prelude::Closure, JsValue};

    use crate::app::{dom::Dom, vdom, widgets::Widget};

    pub(super) struct Element {
        pub(crate) type_: vdom::ElementType,
        pub(crate) props: HashMap<String, JsValue>, // TODO: make this &'static str instead of String?
        pub(crate) event_listeners: Vec<(&'static str, Closure<dyn Fn(JsValue)>)>,
        pub(crate) children: Vec<Node>,
    }
    pub(super) enum Node {
        Element(Element),
        Text(String),
    }

    impl Element {
        pub(super) fn from_normal_vdom<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Fn(&Data) -> DataAsWidget + Copy + 'static>(
            dom_refcell: &Rc<RefCell<Dom>>,
            data_refcell: &Rc<RefCell<Data>>,
            other_elem: vdom::Element<Data>,
            to_widget: ToWidget,
        ) -> Element {
            Element {
                type_: other_elem.type_,
                props: other_elem.props,
                event_listeners: other_elem
                    .event_listeners
                    .into_iter()
                    .map(|(event, event_listener)| {
                        (
                            event,
                            Closure::<dyn Fn(JsValue)>::new({
                                let data_refcell = Rc::clone(data_refcell);
                                let dom_refcell = Rc::clone(dom_refcell);
                                move |event| {
                                    event_listener(event, &mut data_refcell.borrow_mut());
                                    let new_vdom = Element::from_normal_vdom(&dom_refcell, &data_refcell, to_widget(&data_refcell.borrow()).to_vdom(), to_widget);
                                    dom_refcell.borrow_mut().update(&dom_refcell, &data_refcell, new_vdom, to_widget);
                                }
                            }),
                        )
                    })
                    .collect(),
                children: other_elem.children.into_iter().map(|other_child| Element::convert_node(dom_refcell, data_refcell, other_child, to_widget)).collect(),
            }
        }
        fn convert_node<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Copy + Fn(&Data) -> DataAsWidget + 'static>(
            dom_refcell: &Rc<RefCell<Dom>>,
            data_refcell: &Rc<RefCell<Data>>,
            other_node: vdom::Node<Data>,
            to_widget: ToWidget,
        ) -> Node {
            match other_node {
                vdom::Node::Element(e) => Node::Element(Element::from_normal_vdom(dom_refcell, data_refcell, e, to_widget)),
                vdom::Node::Text(t) => Node::Text(t),
            }
        }
    }
}

// the dom and the vdom that created it
pub(crate) struct Dom {
    dom: web_sys::Element,
    document: web_sys::Document,
    vdom: vdom_with_closures::Element,
}

// TODO: clean up this whole module
// (especially clean up lifetimes because this seems like it would leak in ways i dont want)

impl Dom {
    fn new_empty(document: web_sys::Document, parent: &web_sys::Node) -> Dom {
        let vdom = vdom_with_closures::Element { type_: vdom::ElementType::Div, props: HashMap::new(), event_listeners: Vec::new(), children: Vec::new() };
        let dom = document.create_element(vdom::ElementType::Div.stringify()).unwrap(); // TODO: handle errors properly
        parent.append_child(&dom).unwrap(); // TODO: handle this error properly
        Dom { dom, document, vdom }
    }

    pub(crate) fn new<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Fn(&Data) -> DataAsWidget + Copy + 'static>(
        data: &Rc<RefCell<Data>>,
        document: web_sys::Document,
        parent: &web_sys::Node,
        to_widget: ToWidget,
    ) -> Rc<RefCell<Dom>> {
        let dom = Rc::new(RefCell::new(Self::new_empty(document, parent)));
        let vdom = vdom_with_closures::Element::from_normal_vdom(&dom, data, to_widget(&data.borrow()).to_vdom(), to_widget);
        dom.borrow_mut().update(&dom, data, vdom, to_widget);
        dom
    }

    fn update<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Fn(&Data) -> DataAsWidget + Copy + 'static>(
        &mut self,
        dom_refcell: &Rc<RefCell<Dom>>,
        data: &Rc<RefCell<Data>>,
        mut new_vdom: vdom_with_closures::Element,
        to_widget: ToWidget,
    ) {
        self.dom = Self::update_dom(dom_refcell, data, to_widget, &self.document, &self.dom, &self.vdom, &mut new_vdom);
        self.vdom = new_vdom;
    }

    fn make_child<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Fn(&Data) -> DataAsWidget + Copy + 'static>(
        dom_refcell: &Rc<RefCell<Dom>>,
        data: &Rc<RefCell<Data>>,
        to_widget: ToWidget,
        document: &web_sys::Document,
        child: &mut vdom_with_closures::Node,
        parent: &web_sys::Element,
    ) {
        // TODO: handle all the errors properly (all the unwraps) (maybe also grep over all code too?)
        match child {
            vdom_with_closures::Node::Element(e) => {
                parent.append_child(&Self::make_dom(dom_refcell, data, to_widget, document, e)).unwrap();
            }
            vdom_with_closures::Node::Text(t) => {
                parent.append_child(&document.create_text_node(t)).unwrap();
            }
        }
    }
    fn make_dom<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Fn(&Data) -> DataAsWidget + Copy + 'static>(
        dom_refcell: &Rc<RefCell<Dom>>,
        data: &Rc<RefCell<Data>>,
        to_widget: ToWidget,
        document: &web_sys::Document,
        vdom: &mut vdom_with_closures::Element,
    ) -> web_sys::Element {
        // TODO: handle all the errors properly (all the unwraps)
        let elem = document.create_element(vdom.type_.stringify()).unwrap();

        for (prop_name, prop_value) in &vdom.props {
            js_sys::Reflect::set(&elem, &prop_name.into(), prop_value).unwrap();
        }

        Self::add_event_listeners(&elem, &vdom.event_listeners);

        for child in &mut vdom.children {
            Self::make_child(dom_refcell, data, to_widget, document, child, &elem);
        }

        elem
    }

    fn update_child<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Fn(&Data) -> DataAsWidget + Copy + 'static>(
        dom_refcell: &Rc<RefCell<Dom>>,
        data: &Rc<RefCell<Data>>,
        to_widget: ToWidget,
        document: &web_sys::Document,
        dom: web_sys::Node,
        old_vdom: &vdom_with_closures::Node,
        new_vdom: &mut vdom_with_closures::Node,
    ) {
        match (old_vdom, new_vdom) {
            (vdom_with_closures::Node::Element(old_vdom), vdom_with_closures::Node::Element(new_vdom)) => {
                Self::update_dom(dom_refcell, data, to_widget, document, dom.dyn_ref().expect("if both old vdom and new vdom are elements, the current dom should be element"), old_vdom, new_vdom);
            }
            (vdom_with_closures::Node::Text(_), vdom_with_closures::Node::Text(new_text)) => {
                dom.set_node_value(Some(new_text));
            }
            (_, vdom_with_closures::Node::Text(new_text)) => {
                // TODO: handle error properly
                let parent_node = dom.parent_node().expect("node should have parent");
                parent_node.replace_child(&document.create_text_node(new_text), &dom).unwrap();
            }
            (_, vdom_with_closures::Node::Element(new_vdom)) => {
                // TODO: handle error properly
                let parent_node = dom.parent_node().expect("node should have parent");
                parent_node.replace_child(&Self::make_dom(dom_refcell, data, to_widget, document, new_vdom), &dom).unwrap();
            }
        }
    }
    fn update_dom<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Fn(&Data) -> DataAsWidget + Copy + 'static>(
        dom_refcell: &Rc<RefCell<Dom>>,
        data: &Rc<RefCell<Data>>,
        to_widget: ToWidget,
        document: &web_sys::Document,
        dom: &web_sys::Element,
        old_vdom: &vdom_with_closures::Element,
        new_vdom: &mut vdom_with_closures::Element,
    ) -> web_sys::Element {
        if old_vdom.type_ != new_vdom.type_ {
            // TODO: handle error properly
            let made = Self::make_dom(dom_refcell, data, to_widget, document, new_vdom);
            dom.replace_with_with_node_1(&made).unwrap();
            made
        } else {
            // update properties
            // remove old properties
            for old_prop_name in old_vdom.props.keys() {
                if !new_vdom.props.contains_key(old_prop_name) {
                    // TODO: deal with error properly
                    js_sys::Reflect::delete_property(dom, &old_prop_name.into()).unwrap();
                }
            }
            // set new properties
            for (new_prop_name, new_value) in &new_vdom.props {
                // TODO: deal with error properly
                // TODO: only set if value changed?
                js_sys::Reflect::set(dom, &new_prop_name.into(), new_value).unwrap();
            }

            Self::remove_event_listeners(dom, &old_vdom.event_listeners);
            Self::add_event_listeners(dom, &new_vdom.event_listeners);

            // update children
            {
                let child_doms = dom.child_nodes();
                assert_eq!(child_doms.length() as usize, old_vdom.children.len());
                // remove children if there are too many
                if old_vdom.children.len() > new_vdom.children.len() {
                    for remove_i in new_vdom.children.len()..old_vdom.children.len() {
                        let this_node = child_doms.get(remove_i as u32).expect("index should be in range of child list");
                        let parent_node = this_node.parent_node().expect("node should have parent");
                        parent_node.remove_child(&this_node).unwrap(); // TODO: handle error properly
                    }
                }
                // update the children
                for (index, (old_vdom, new_vdom)) in old_vdom.children.iter().zip(&mut new_vdom.children).enumerate() {
                    let dom = child_doms.get(index as u32).expect("index should be in range of child list");
                    Self::update_child(dom_refcell, data, to_widget, document, dom, old_vdom, new_vdom);
                }
                // add the new children if they need to be added
                if old_vdom.children.len() < new_vdom.children.len() {
                    for vdom_to_add in new_vdom.children.iter_mut().skip(old_vdom.children.len()) {
                        Self::make_child(dom_refcell, data, to_widget, document, vdom_to_add, dom);
                    }
                }
            }

            dom.clone() // TODO: figure out a better way than to use clone
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
            dom.remove_event_listener_with_callback(event, listener.as_ref().unchecked_ref()).unwrap();
            // TODO: handle error correctly instead of unwrapping
        }
    }
}
