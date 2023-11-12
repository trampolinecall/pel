#[macro_use]
pub(crate) mod fixed_amount;
pub(crate) mod homogeneous;

// TODO: decide on a better name for this?
pub(crate) mod _layout {
    use crate::visualizer::{
        vdom,
        widgets::flex::{Direction, ItemSettings},
    };

    #[inline]
    pub(crate) fn make_flexbox<Data>(direction: Direction, children: Vec<(ItemSettings, vdom::Element<Data>)>) -> vdom::Element<Data> {
        vdom::Element {
            type_: vdom::ElementType::Div,
            props: std::iter::once((
                "style".to_string(),
                format!(
                    "display: flex; flex-direction: {};",
                    match direction {
                        Direction::Horizontal => {
                            "horizontal"
                        }
                        Direction::Vertical => {
                            "vertical"
                        }
                    }
                )
                .into(),
            ))
            .collect(),
            event_listeners: Vec::new(),
            children: children
                .into_iter()
                .map(|(settings, mut child)| {
                    match settings {
                        ItemSettings::Fixed => {} // TODO: do this correctly
                        ItemSettings::Flex(grow_proportion) => {
                            // TODO: do this better
                            if !child.props.contains_key("style") {
                                child.props.insert("style".to_string(), "".into());
                            }
                            let style_value = child.props.get_mut("style").expect("style property should exist after being created");
                            *style_value = (style_value.as_string().expect("style property should be a string") + &format!("flex-grow: {grow_proportion};")).into();
                        }
                    }
                    vdom::ElementChild::Element(child)
                })
                .collect(),
        }
    }
    /* TODO: REMOVE this whole module?
    #[inline]
    pub(crate) fn animated_settings(settings: Animated<ItemSettings>) -> ItemSettings {
        match settings.get() {
            AnimatedValue::Steady(s) => *s,
            AnimatedValue::Animating { before: ItemSettings::Flex(before_flex), after: ItemSettings::Flex(after_flex), amount } => ItemSettings::Flex(before_flex.lerp(after_flex, amount)),
            AnimatedValue::Animating { before: _, after, amount: _ } => *after,
        }
    }
    */
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum ItemSettings {
    Fixed,
    Flex(f32),
}

#[derive(Copy, Clone)]
pub(crate) enum Direction {
    Horizontal,
    Vertical,
}

/* TODO: REMOVE?
impl Direction {
    pub(crate) fn make_vector_in_direction<T>(&self, major_component: T, minor_component: T) -> graphics::Vector2<T> {
        match self {
            Direction::Horizontal => graphics::Vector2::new(major_component, minor_component),
            Direction::Vertical => graphics::Vector2::new(minor_component, major_component),
        }
    }

    pub(crate) fn take_major_component<T>(&self, v: graphics::Vector2<T>) -> T {
        match self {
            Direction::Horizontal => v.x,
            Direction::Vertical => v.y,
        }
    }

    pub(crate) fn take_minor_component<T>(&self, v: graphics::Vector2<T>) -> T {
        match self {
            Direction::Horizontal => v.y,
            Direction::Vertical => v.x,
        }
    }
}
*/
