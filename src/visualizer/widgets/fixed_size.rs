use std::marker::PhantomData;

use crate::visualizer::{dom, graphics, widgets::Widget};

// TODO: REMOVE this module?

pub(crate) struct FixedSize<Data, Child: Widget<Data>> {
    child: Child,
    size: graphics::Vector2f,

    _phantom: PhantomData<fn(&mut Data)>,
}

impl<Data, Child: Widget<Data>> FixedSize<Data, Child> {
    pub(crate) fn new(child: Child, size: graphics::Vector2f) -> Self {
        Self { child, _phantom: PhantomData, size }
    }
}

impl<Data, Child: Widget<Data>> Widget<Data> for FixedSize<Data, Child> {
    fn to_vdom(self) -> dom::Element<Data> {
        dom::Element {
            type_: dom::ElementType::Div,
            // TODO: do something about .into_iter().collect() (and remember to change it across the whole project)
            props: vec![("style".to_string(), format!("width: {}px; height: {}px;", self.size.x, self.size.y).into())].into_iter().collect(),
            event_listeners: Vec::new(),
            children: vec![dom::ElementChild::Element(self.child.to_vdom())],
        }
    }
}
