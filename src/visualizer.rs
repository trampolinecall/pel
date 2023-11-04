pub(crate) mod event;
pub(crate) mod graphics;
pub(crate) mod layout;
pub(crate) mod lens;
pub(crate) mod render_object;
#[macro_use]
pub(crate) mod widgets;

use sfml::{
    graphics::{RenderTarget, RenderWindow},
    window::{Event, Style},
};

use crate::visualizer::{
    event::{GeneralEvent, TargetedEvent},
    graphics::{Fonts, GraphicsContext},
    layout::SizeConstraints,
    render_object::{RenderObject, RenderObjectIdMaker},
    widgets::Widget,
};

pub(crate) fn run<Model, ModelAsWidget: Widget<Model>>(window_name: &'static str, window_size: (u32, u32), mut model: Model, model_to_widget: impl Fn(&Model) -> ModelAsWidget) {
    let mut id_maker = RenderObjectIdMaker::new();
    let graphics_context = {
        let fonts = {
            // TODO: don't panic?
            let text_font_handle = font_kit::source::SystemSource::new()
                .select_best_match(&[font_kit::family_name::FamilyName::SansSerif, font_kit::family_name::FamilyName::Serif], &font_kit::properties::Properties::new())
                .expect("could not find appropriate text font font");
            let text_font = match text_font_handle {
                font_kit::handle::Handle::Path { path, font_index: _ } => graphics::Font::from_file(&path.to_string_lossy()).expect("could not load font"), // TODO: figure out how to handle font_index
                font_kit::handle::Handle::Memory { bytes: _, font_index: _ } => unimplemented!("loading font from memory"),
            };

            let monospace_font_handle = font_kit::source::SystemSource::new()
                .select_best_match(&[font_kit::family_name::FamilyName::Monospace], &font_kit::properties::Properties::new())
                .expect("could not find appropriate monospace font");
            let monospace_font = match monospace_font_handle {
                font_kit::handle::Handle::Path { path, font_index: _ } => graphics::Font::from_file(&path.to_string_lossy()).expect("could not load font"), // TODO: figure out how to handle font_index
                font_kit::handle::Handle::Memory { bytes: _, font_index: _ } => unimplemented!("loading font from memory"),
            };

            Fonts { text_font, monospace_font }
        };

        GraphicsContext { default_render_context_settings: sfml::window::ContextSettings { antialiasing_level: 0, ..Default::default() }, fonts }
    };

    let mut render_object = model_to_widget(&model).to_render_object(&mut id_maker);

    let mut window = RenderWindow::new(window_size, window_name, Style::DEFAULT, &graphics_context.default_render_context_settings);
    window.set_vertical_sync_enabled(true);

    while window.is_open() {
        // TODO: this doesnt seem right
        let view_top_left = graphics::Vector2f::new(0.0, 0.0);

        // events
        while let Some(event) = window.poll_event() {
            match event {
                // TODO: put these in the event handler with everything else
                Event::Closed => window.close(),
                Event::Resized { width, height } => {
                    // update the view to the new size of the window
                    let visible_area = graphics::FloatRect::new(0.0, 0.0, width as f32, height as f32);
                    window.set_view(&sfml::graphics::View::from_rect(visible_area));
                }

                sfml::window::Event::MouseButtonPressed { button, x, y } => {
                    let mouse_position = graphics::Vector2f::new(x as f32, y as f32); // TODO: clean up casts (also clean up in rest of module too)
                    let hovered = render_object.find_hover(view_top_left, mouse_position);
                    if let Some(hovered) = hovered {
                        match button {
                            sfml::window::mouse::Button::Left => render_object.send_targeted_event(view_top_left, &mut model, hovered, TargetedEvent::LeftMouseDown(mouse_position)),
                            sfml::window::mouse::Button::Right => render_object.send_targeted_event(view_top_left, &mut model, hovered, TargetedEvent::RightMouseDown(mouse_position)),
                            _ => {}
                        }
                    }
                }

                sfml::window::Event::MouseMoved { x, y } => render_object.general_event(view_top_left, &mut model, GeneralEvent::MouseMoved(graphics::Vector2f::new(x as f32, y as f32))), // TODO: change the event to accept 2 i32s

                sfml::window::Event::MouseButtonReleased { button: sfml::window::mouse::Button::Left, x: _, y: _ } => render_object.general_event(view_top_left, &mut model, GeneralEvent::LeftMouseUp),
                sfml::window::Event::MouseButtonReleased { button: sfml::window::mouse::Button::Right, x: _, y: _ } => {
                    render_object.general_event(view_top_left, &mut model, GeneralEvent::RightMouseUp);
                }

                // TODO: proper event dispatch (including reworking the mouse events above)
                sfml::window::Event::KeyPressed { code, alt, ctrl, shift, system } => {
                    render_object.general_event(view_top_left, &mut model, GeneralEvent::KeyPressed { code, alt, ctrl, shift, system });
                }

                _ => {}
            }
        }

        // draw
        window.set_active(true);
        model_to_widget(&model).update_render_object(&mut render_object, &mut id_maker);

        let size_constraints = SizeConstraints { min: graphics::Vector2f::new(0.0, 0.0), max: window.size().as_other() };

        render_object.layout(&graphics_context, size_constraints);

        let mouse_position = window.mouse_position().as_other();
        let hover = render_object.find_hover(view_top_left, mouse_position);

        window.clear(graphics::Color::BLACK);
        render_object.draw(&graphics_context, &mut window, view_top_left, hover);

        window.display();
    }
}
