use std::{cell::RefCell, collections::HashMap, rc::Rc};

use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::js_sys::Reflect;

use crate::visualizer::widgets::Widget;

// the dom, the vdom that created it, and the closures that this dom references
pub(crate) struct ActualDom<Data, DataAsWidget: Widget<Data>, ToWidget: Fn(&Data) -> DataAsWidget + Copy> {
    dom: web_sys::Element,
    vdom: Element<Data>,
    to_widget: ToWidget,
    closures: Vec<Closure<dyn Fn(JsValue)>>,
}

// TODO: it seems a bit strange to have the Data parameter here; figure out a better way?
pub(crate) struct Element<Data: ?Sized> {
    pub(crate) type_: ElementType,
    pub(crate) props: HashMap<String, JsValue>,                                       // TODO: make this &'static str instead of String?
    pub(crate) event_listeners: Vec<(&'static str, Box<dyn Fn(JsValue, &mut Data)>)>, // TODO: do not box closures?
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
// TODO: clean this up
impl<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Fn(&Data) -> DataAsWidget + Copy + 'static> ActualDom<Data, DataAsWidget, ToWidget> {
    fn new_empty(document: &web_sys::Document, parent: &web_sys::Node, to_widget: ToWidget) -> ActualDom<Data, DataAsWidget, ToWidget> {
        let closures = Vec::new();
        let vdom = Element { type_: ElementType::Div, props: HashMap::new(), event_listeners: Vec::new(), children: Vec::new() };
        let dom = document.create_element(ElementType::Div.stringify()).unwrap(); // TODO: handle errors properly
        parent.append_child(&dom).unwrap(); // TODO: handle this error properly
        ActualDom { dom, vdom, to_widget, closures }
    }

    pub(crate) fn new(data: &Rc<RefCell<Data>>, document: &web_sys::Document, parent: &web_sys::Node, to_widget: ToWidget) -> Rc<RefCell<ActualDom<Data, DataAsWidget, ToWidget>>> {
        let vdom = to_widget(&data.borrow()).to_vdom();
        let dom = Rc::new(RefCell::new(Self::new_empty(document, parent, to_widget)));
        dom.borrow_mut().update(&dom, data, document, vdom);
        dom
    }

    pub(crate) fn dom(&self) -> &web_sys::Element {
        &self.dom
    }
    pub(crate) fn update(&mut self, dom_refcell: &Rc<RefCell<ActualDom<Data, DataAsWidget, ToWidget>>>, data: &Rc<RefCell<Data>>, document: &web_sys::Document, mut new_vdom: Element<Data>) {
        self.closures.clear();
        update_dom(&mut self.closures, dom_refcell, data, self.to_widget, document, &self.dom, &self.vdom, &mut new_vdom);
        self.vdom = new_vdom;
    }
}

fn make_child<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Fn(&Data) -> DataAsWidget + Copy + 'static>(
    closures: &mut Vec<Closure<dyn Fn(JsValue)>>,
    dom_refcell: &Rc<RefCell<ActualDom<Data, DataAsWidget, ToWidget>>>,
    data: &Rc<RefCell<Data>>,
    to_widget: ToWidget,
    document: &web_sys::Document,
    child: &mut ElementChild<Data>,
    parent: &web_sys::Element,
) {
    // TODO: handle all the errors properly (all the unwraps) (maybe also grep over all code too?)
    match child {
        ElementChild::Element(e) => {
            parent.append_child(&make_dom(closures, dom_refcell, data, to_widget, document, e)).unwrap();
        }
        ElementChild::Text(t) => {
            parent.append_child(&document.create_text_node(t)).unwrap();
        }
    }
}
fn make_dom<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Fn(&Data) -> DataAsWidget + Copy + 'static>(
    closures: &mut Vec<Closure<dyn Fn(JsValue)>>,
    dom_refcell: &Rc<RefCell<ActualDom<Data, DataAsWidget, ToWidget>>>,
    data: &Rc<RefCell<Data>>,
    to_widget: ToWidget,
    document: &web_sys::Document,
    vdom: &mut Element<Data>,
) -> web_sys::Element {
    // TODO: handle all the errors properly (all the unwraps)
    let elem = document.create_element(vdom.type_.stringify()).unwrap();

    for (prop_name, prop_value) in &vdom.props {
        Reflect::set(&elem, &prop_name.into(), prop_value).unwrap();
    }

    // empty the vdom's event listeners, but that should be fine because the only thing that this vdom is going to be used for is diffing against the next vdom, and event listeners are not compared in the update process so this one doesn't need its event listeners
    add_event_listeners(closures, dom_refcell, data, to_widget, document, &elem, std::mem::take(&mut vdom.event_listeners));

    for child in &mut vdom.children {
        make_child(closures, dom_refcell, data, to_widget, document, child, &elem);
    }

    elem
}

fn update_child<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Fn(&Data) -> DataAsWidget + Copy + 'static>(
    closures: &mut Vec<Closure<dyn Fn(JsValue)>>,
    dom_refcell: &Rc<RefCell<ActualDom<Data, DataAsWidget, ToWidget>>>,
    data: &Rc<RefCell<Data>>,
    to_widget: ToWidget,
    document: &web_sys::Document,
    dom: web_sys::Node,
    old_vdom: &ElementChild<Data>,
    new_vdom: &mut ElementChild<Data>,
) {
    match (old_vdom, new_vdom) {
        (ElementChild::Element(old_vdom), ElementChild::Element(new_vdom)) => {
            update_dom(closures, dom_refcell, data, to_widget, document, dom.dyn_ref().expect("if both old vdom and new vdom are elements, the current dom should be element"), old_vdom, new_vdom);
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
            parent_node.replace_child(&make_dom(closures, dom_refcell, data, to_widget, document, new_vdom), &dom).unwrap();
        }
    }
}
// this method empties all of the new_vdom's event listeners (including ones in child nodes)
fn update_dom<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Fn(&Data) -> DataAsWidget + Copy + 'static>(
    closures: &mut Vec<Closure<dyn Fn(JsValue)>>,
    dom_refcell: &Rc<RefCell<ActualDom<Data, DataAsWidget, ToWidget>>>,
    data: &Rc<RefCell<Data>>,
    to_widget: ToWidget,
    document: &web_sys::Document,
    dom: &web_sys::Element,
    old_vdom: &Element<Data>,
    new_vdom: &mut Element<Data>,
) {
    if old_vdom.type_ != new_vdom.type_ {
        // TODO: handle error properly
        dom.replace_with_with_node_1(&make_dom(closures, dom_refcell, data, to_widget, document, new_vdom)).unwrap();
    } else {
        // update properties
        // remove old properties
        for old_prop_name in old_vdom.props.keys() {
            if !new_vdom.props.contains_key(old_prop_name) {
                // TODO: deal with error properly
                Reflect::delete_property(dom, &old_prop_name.into()).unwrap();
            }
        }
        // set new properties
        for (new_prop_name, new_value) in &new_vdom.props {
            // TODO: deal with error properly
            // TODO: only set if value changed?
            Reflect::set(dom, &new_prop_name.into(), new_value).unwrap();
        }

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
                update_child(closures, dom_refcell, data, to_widget, document, dom, old_vdom, new_vdom);
            }
            // add the new children if they need to be added
            if old_vdom.children.len() < new_vdom.children.len() {
                for vdom_to_add in new_vdom.children.iter_mut().skip(old_vdom.children.len()) {
                    make_child(closures, dom_refcell, data, to_widget, document, vdom_to_add, dom);
                }
            }
        }

        // clear event listeners by replacing with a clone
        // (this goes at the end because the original dom node should not be modified after the clone is made)
        // TODO: find a better way to clear event listeners?
        let parent_node = dom.parent_node().expect("node should have parent");
        let clone = dom.clone_node_with_deep(true).unwrap(); // TODO: handle error properly
        parent_node.replace_child(&clone, dom).unwrap(); // TODO: handle error properly
        add_event_listeners(closures, dom_refcell, data, to_widget, document, &clone, std::mem::take(&mut new_vdom.event_listeners));
        // also empty the new_vdom's event listeners; for an explanation of why this is fine, see the "empty the vdom's event listeners" comment above
    }
}

fn add_event_listeners<Data: 'static, DataAsWidget: Widget<Data> + 'static, ToWidget: Fn(&Data) -> DataAsWidget + Copy + 'static>(
    closures: &mut Vec<Closure<dyn Fn(JsValue)>>,
    dom_refcell: &Rc<RefCell<ActualDom<Data, DataAsWidget, ToWidget>>>,
    data: &Rc<RefCell<Data>>,
    to_widget: ToWidget,
    document: &web_sys::Document,
    node: &web_sys::Node,
    event_listeners: Vec<(&str, Box<dyn Fn(JsValue, &mut Data)>)>,
) {
    for (event, listener) in event_listeners {
        let closure = Closure::<dyn Fn(JsValue)>::new({
            let data = Rc::clone(data);
            let dom_refcell = Rc::clone(dom_refcell);
            let document = document.clone();
            move |event| {
                listener(event, &mut data.borrow_mut());
                dom_refcell.borrow_mut().update(&dom_refcell, &data, &document, to_widget(&data.borrow()).to_vdom());
            }
        });
        node.add_event_listener_with_callback(event, closure.as_ref().unchecked_ref()).unwrap(); // TODO: handle error correctly
        closures.push(closure);
    }
}
