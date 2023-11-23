// the syntax of this macro is a little strange but im not sure of a better way
// the goal was to make it parseable as a struct so that it can be automatically formatted by rustfmt
macro_rules! flex_ty {
    (#[horizontal] $($thingies:tt)+) => {
        parse_struct! (then flex_ty!(impl, $crate::app::widgets::flex::Direction::Horizontal), $($thingies)+)
    };
    (#[vertical] $($thingies:tt)*) => {
        parse_struct! (then flex_ty!(impl, $crate::app::widgets::flex::Direction::Vertical), $($thingies)+)
    };

    (impl, $dontcare:path, struct {
        attrs: $dontcare2:tt,
        vis: $dontcare3:tt,
        name: $dontcare4:ident,
        generics: $dontcare5:tt,
        where: $dontcare6:tt,
        kind: unitary,
        fields: $dontcare7:tt,
        num_fields: $dontcare8:tt,
    }) => { compile_error!("this macro can only work with a record struct") };
    (impl, $dontcare:path, struct {
        attrs: $dontcare2:tt,
        vis: $dontcare3:tt,
        name: $dontcare4:ident,
        generics: $dontcare5:tt,
        where: $dontcare6:tt,
        kind: tuple,
        fields: $dontcare7:tt,
        num_fields: $dontcare8:tt,
    }) => { compile_error!("this macro can only work with a record struct") };

    (impl, $direction:path, struct {
        attrs: [$(#$attrs:meta)*],
        vis: ($vis:vis),
        name: $name:ident,
        generics: {
            constr: [$($($constr:tt)+),* $(,)?],
            params: $dontcare:tt,
            ltimes: $dontcare2:tt,
            tnames: $dontcare3:tt,
        },
        where: { preds: [$($($wheres:tt)+),* $(,)?] },
        kind: record,
        fields: [
            $(
                {
                    ord: $dontcare4:tt,
                    attrs: [$(#$field_attr:meta)*],
                    vis: ($field_vis:vis),
                    ty: $field_ty:ty,
                    name: $field_name:ident,
                },
             )*
        ],
        num_fields: $dontcare5:tt,
    }) => {
        {
            // TODO: handle extra fields that do not go into layout
            $(#$attrs)*
            $vis struct $name <$($($constr)+),*> where $($($wheres)+),* {
                $(
                    $(#$field_attr)*
                    $field_vis $field_name: $field_ty,
                )*
            }
            /* TODO: REMOVE
            #[allow(non_camel_case_types)]
            struct ContainerRenderObject<Data, $($name: $crate::app::render_object::RenderObject<Data>),*> {
                own_size: graphics::Vector2f,
                $(
                    $name: ($crate::app::render_object::animated::Animated<$crate::app::widgets::flex::ItemSettings>, graphics::Vector2f, $name),
                )*

                _phantom: ::std::marker::PhantomData<fn(&mut Data)>,
            }
            */

            #[allow(non_camel_case_types)]
            impl<Data, $($name: Widget<Data>),*> Widget<Data> for Container<Data, $($name),*> {
                fn to_vdom(self) -> $crate::app::vdom::Element<Data> {
                    $crate::app::widgets::flex::_layout::make_flexbox($direction, vec![$((self.$name.0, self.$name.1.to_vdom())),*])
                }
            }
            /* TODO: REMOVE
            #[allow(non_camel_case_types)]
            impl<Data, $($name: $crate::app::render_object::RenderObject<Data>),*> $crate::app::render_object::RenderObject<Data> for ContainerRenderObject<Data, $($name),*> {
                fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: $crate::app::layout::SizeConstraints) {
                    // lay out fixed elements and count up total flex scaling factors
                    let mut total_flex_scale = 0.0;
                    let mut major_size_left = $direction.take_major_component(sc.max);
                    $(
                        {
                            let (settings, _, ref mut child) = self.$name;
                            $crate::app::widgets::flex::_layout::first_phase_step(graphics_context, sc, $direction, &mut total_flex_scale, &mut major_size_left, $crate::app::widgets::flex::_layout::animated_settings(settings), child);
                        }
                    )*

                    // lay out all of the flex children
                    $(
                        {
                            let (settings, _, ref mut child) = self.$name;
                            $crate::app::widgets::flex::_layout::second_phase_step(graphics_context, sc, $direction, total_flex_scale, major_size_left, $crate::app::widgets::flex::_layout::animated_settings(settings), child);
                        }
                    )*

                    // assign each of the offsets and calcaulte own_size
                    let mut major_offset = 0.0;
                    let mut max_minor_size = 0.0;
                    $(
                        #[allow(unused_assignments)]
                        {
                            let (_, ref mut offset, ref mut child) = self.$name;
                            *offset = $crate::app::widgets::flex::_layout::third_phase_step($direction, &mut major_offset, &mut max_minor_size, child);
                        }
                    )*
                    self.own_size = sc.clamp_size($direction.make_vector_in_direction(major_offset, max_minor_size));
                }

                fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<$crate::app::render_object::RenderObjectId>) {
                    $(
                        {
                            let (_, offset, child) = &self.$name;
                            child.draw(graphics_context, target, top_left + *offset, hover);
                        }
                    )*
                }

                fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<$crate::app::render_object::RenderObjectId> {
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

                fn send_targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, target: $crate::app::render_object::RenderObjectId, event: $crate::app::event::TargetedEvent) {
                    $(
                        self.$name.2.send_targeted_event(top_left + self.$name.1, data, target, event);
                    )*
                }

                fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: $crate::app::event::TargetedEvent) {}
                fn general_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, event: $crate::app::event::GeneralEvent) {
                    $(
                        self.$name.2.general_event(top_left + self.$name.1, data, event);
                     )*
                }
            }
            */

            Container {
                $(
                    $name: ($settings, $e),
                )*
                _phantom: ::std::marker::PhantomData,
            }
        }
    };
}

macro_rules! flex {
    (horizontal { $($rest:tt)* }) => {
        flex!($crate::app::widgets::flex::Direction::Horizontal { $($rest)* })
    };
    (vertical { $($rest:tt)* }) => {
        flex!($crate::app::widgets::flex::Direction::Vertical { $($rest)* })
    };
    ($direction:path { $( $name:ident: ($settings:expr, $e:expr) ),* $(,)? }) => {
        {
            #[allow(non_camel_case_types)]
            struct Container<Data, $($name: Widget<Data>),*> {
                $(
                    $name: ($crate::app::widgets::flex::ItemSettings, $name),
                )*
                _phantom: ::std::marker::PhantomData<fn(&mut Data)>,
            }
            /* TODO: REMOVE
            #[allow(non_camel_case_types)]
            struct ContainerRenderObject<Data, $($name: $crate::app::render_object::RenderObject<Data>),*> {
                own_size: graphics::Vector2f,
                $(
                    $name: ($crate::app::render_object::animated::Animated<$crate::app::widgets::flex::ItemSettings>, graphics::Vector2f, $name),
                )*

                _phantom: ::std::marker::PhantomData<fn(&mut Data)>,
            }
            */

            #[allow(non_camel_case_types)]
            impl<Data, $($name: Widget<Data>),*> Widget<Data> for Container<Data, $($name),*> {
                fn to_vdom(self) -> $crate::app::vdom::Element<Data> {
                    $crate::app::widgets::flex::_layout::make_flexbox($direction, vec![$((self.$name.0, self.$name.1.to_vdom())),*])
                }
            }
            /* TODO: REMOVE
            #[allow(non_camel_case_types)]
            impl<Data, $($name: $crate::app::render_object::RenderObject<Data>),*> $crate::app::render_object::RenderObject<Data> for ContainerRenderObject<Data, $($name),*> {
                fn layout(&mut self, graphics_context: &graphics::GraphicsContext, sc: $crate::app::layout::SizeConstraints) {
                    // lay out fixed elements and count up total flex scaling factors
                    let mut total_flex_scale = 0.0;
                    let mut major_size_left = $direction.take_major_component(sc.max);
                    $(
                        {
                            let (settings, _, ref mut child) = self.$name;
                            $crate::app::widgets::flex::_layout::first_phase_step(graphics_context, sc, $direction, &mut total_flex_scale, &mut major_size_left, $crate::app::widgets::flex::_layout::animated_settings(settings), child);
                        }
                    )*

                    // lay out all of the flex children
                    $(
                        {
                            let (settings, _, ref mut child) = self.$name;
                            $crate::app::widgets::flex::_layout::second_phase_step(graphics_context, sc, $direction, total_flex_scale, major_size_left, $crate::app::widgets::flex::_layout::animated_settings(settings), child);
                        }
                    )*

                    // assign each of the offsets and calcaulte own_size
                    let mut major_offset = 0.0;
                    let mut max_minor_size = 0.0;
                    $(
                        #[allow(unused_assignments)]
                        {
                            let (_, ref mut offset, ref mut child) = self.$name;
                            *offset = $crate::app::widgets::flex::_layout::third_phase_step($direction, &mut major_offset, &mut max_minor_size, child);
                        }
                    )*
                    self.own_size = sc.clamp_size($direction.make_vector_in_direction(major_offset, max_minor_size));
                }

                fn draw(&self, graphics_context: &graphics::GraphicsContext, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<$crate::app::render_object::RenderObjectId>) {
                    $(
                        {
                            let (_, offset, child) = &self.$name;
                            child.draw(graphics_context, target, top_left + *offset, hover);
                        }
                    )*
                }

                fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<$crate::app::render_object::RenderObjectId> {
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

                fn send_targeted_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, target: $crate::app::render_object::RenderObjectId, event: $crate::app::event::TargetedEvent) {
                    $(
                        self.$name.2.send_targeted_event(top_left + self.$name.1, data, target, event);
                    )*
                }

                fn targeted_event(&mut self, _: graphics::Vector2f, _: &mut Data, _: $crate::app::event::TargetedEvent) {}
                fn general_event(&mut self, top_left: graphics::Vector2f, data: &mut Data, event: $crate::app::event::GeneralEvent) {
                    $(
                        self.$name.2.general_event(top_left + self.$name.1, data, event);
                     )*
                }
            }
            */

            Container {
                $(
                    $name: ($settings, $e),
                )*
                _phantom: ::std::marker::PhantomData,
            }
        }
    };
}
