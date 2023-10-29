use crate::visualizer::graphics;

pub(crate) enum Direction {
    Horizontal,
    Vertical,
}

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

pub(crate) enum FlexItemSettings {
    Fixed,
    Flex(f32),
}

macro_rules! flex {
    (horizontal $($rest:tt)*) => {
        flex!($crate::visualizer::widgets::fixed_amount_flex::Direction::Horizontal; $($rest)*)
    };
    (vertical $($rest:tt)*) => {
        flex!($crate::visualizer::widgets::fixed_amount_flex::Direction::Vertical; $($rest)*)
    };
    ($direction:expr; $( $name:ident : $settings:expr, $e:expr ),* $(,)?) => {
        {
            use std::marker::PhantomData;

            use $crate::visualizer::{
                graphics, layout, event,
                render_object::{RenderObject, RenderObjectId, RenderObjectIdMaker},
                widgets::{fixed_amount_flex::FlexItemSettings, Widget},
            };

            #[allow(non_camel_case_types)]
            struct Container<Data, $($name: Widget<Data>),*> {
                $(
                    $name: (FlexItemSettings, $name),
                )*
                _phantom: ::std::marker::PhantomData<fn(&mut Data)>,
            }
            #[allow(non_camel_case_types)]
            struct ContainerRenderObject<Data, $($name: RenderObject<Data>),*> {
                own_size: graphics::Vector2f,
                $(
                    $name: (FlexItemSettings, graphics::Vector2f, $name),
                )*

                _phantom: ::std::marker::PhantomData<fn(&mut Data)>,
            }

            #[allow(non_camel_case_types)]
            impl<Data, $($name: Widget<Data>),*> Widget<Data> for Container<Data, $($name),*> {
                type Result = ContainerRenderObject<Data, $(<$name as Widget<Data>>::Result),*>;

                fn to_render_object(self, id_maker: &mut RenderObjectIdMaker) -> Self::Result {
                    ContainerRenderObject {
                        own_size: graphics::Vector2f::new(0.0, 0.0),
                        $(
                            $name: (self.$name.0, graphics::Vector2f::new(0.0, 0.0), self.$name.1.to_render_object(id_maker)),
                        )*
                        _phantom: PhantomData,
                    }
                }

                fn update_render_object(self, render_object: &mut Self::Result, id_maker: &mut RenderObjectIdMaker) {
                    $(
                        render_object.$name.0 = self.$name.0;
                        self.$name.1.update_render_object(&mut render_object.$name.2, id_maker);
                    )*
                }
            }
            #[allow(non_camel_case_types)]
            impl<Data, $($name: RenderObject<Data>),*> RenderObject<Data> for ContainerRenderObject<Data, $($name),*> {
                fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: layout::SizeConstraints) {
                    // lay out fixed elements and count up total flex scaling factors
                    let mut total_flex_scale = 0.0;
                    let mut major_size_left = $direction.take_major_component(sc.max);
                    $(
                        {
                            let (ref settings, _, ref mut child) = self.$name;
                            match settings {
                                FlexItemSettings::Fixed => {
                                    child.layout(graphics_context, sc);
                                    major_size_left -= $direction.take_major_component(child.size());
                                }
                                FlexItemSettings::Flex(scale) => {
                                    total_flex_scale += scale;
                                }
                            }
                        }
                    )*

                    // lay out all of the flex children
                    $(
                        {
                            let (ref settings, _, ref mut child) = self.$name;
                            if let FlexItemSettings::Flex(scale) = settings {
                                let child_sc = layout::SizeConstraints { min: graphics::Vector2f::new(0.0, 0.0), max: $direction.make_vector_in_direction(scale / total_flex_scale * major_size_left, $direction.take_minor_component(sc.max)) };
                                child.layout(graphics_context, child_sc);
                            }
                        }
                    )*

                    // assign each of the offsets and calcaulte own_size
                    let mut current_offset = 0.0;
                    let mut max_minor_size = 0.0;
                    $(
                        #[allow(unused_assignments)]
                        {
                            let (_, ref mut offset, ref mut child) = self.$name;
                            *offset = $direction.make_vector_in_direction(current_offset, 0.0);
                            current_offset += $direction.take_major_component(child.size());
                            let child_minor_size = $direction.take_minor_component(child.size());
                            max_minor_size = if child_minor_size > max_minor_size { child_minor_size } else { max_minor_size };
                        }
                    )*
                }

                fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<RenderObjectId>) {
                    $(
                        {
                            let (_, offset, child) = &self.$name;
                            child.draw(graphics_context, target, top_left + *offset, hover);
                        }
                    )*
                }

                fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<RenderObjectId> {
                    None
                        $(
                            .or({
                                let (_, offset, child) = &self.$name;
                                child.find_hover(top_left + *offset, mouse)
                            })
                        )*
                }

                fn size(&self) -> graphics::Vector2f {
                    self.own_size
                }

                fn send_targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, target: RenderObjectId, event: event::TargetedEvent) {
                    $(
                        self.$name.2.send_targeted_event(top_left + self.$name.1, data, target, event);
                    )*
                }

                fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: event::TargetedEvent) {}
                fn general_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, event: event::GeneralEvent) {
                    $(
                        self.$name.2.general_event(top_left + self.$name.1, data, event);
                     )*
                }
            }

            Container {
                $(
                    $name: ($settings, $e),
                )*
                _phantom: ::std::marker::PhantomData,
            }
        }
    };
}
