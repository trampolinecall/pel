mod event;
pub(crate) mod graphics;
mod layout;
pub(crate) mod lens;
pub(crate) mod render_object;
pub(crate) mod view;
pub(crate) mod widgets;

use sfml::{
    graphics::{RenderTarget, RenderWindow},
    window::{Event, Style},
};

use crate::visualizer::{
    event::{GeneralEvent, TargetedEvent},
    layout::SizeConstraints,
    render_object::{RenderObject, RenderObjectIdMaker},
    view::View,
    widgets::Widget,
};

pub(crate) fn run(window_name: &'static str, window_size: (u32, u32), mut model: impl View) {
    let mut render_object = {
        let mut id_maker = RenderObjectIdMaker::new();
        model.to_widget().to_render_object(&mut id_maker)
    };

    let mut window = RenderWindow::new(window_size, window_name, Style::DEFAULT, &graphics::default_render_context_settings());
    window.set_vertical_sync_enabled(true);

    while window.is_open() {
        // TODO: don't duplicate view_top_left?
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
                            sfml::window::mouse::Button::Left => render_object.send_targeted_event(&mut model, hovered, TargetedEvent::LeftMouseDown(mouse_position)),
                            sfml::window::mouse::Button::Right => render_object.send_targeted_event(&mut model, hovered, TargetedEvent::RightMouseDown(mouse_position)),
                            _ => {}
                        }
                    }
                }

                sfml::window::Event::MouseMoved { x, y } => render_object.general_event(&mut model, GeneralEvent::MouseMoved(graphics::Vector2f::new(x as f32, y as f32))), // TODO: change the event to accept 2 i32s

                sfml::window::Event::MouseButtonReleased { button: sfml::window::mouse::Button::Left, x: _, y: _ } => render_object.general_event(&mut model, GeneralEvent::LeftMouseUp),

                _ => {}
            }
        }

        // update
        /* TODO: update
        let mut time_since_last_update = std::time::Instant::now() - todo!("last_update");
        model.update(time_since_last_update);
        */

        // draw
        window.set_active(true);
        model.to_widget().update_render_object(&mut render_object);

        let view_top_left = graphics::Vector2f::new(0.0, 0.0);
        let size_constraints = SizeConstraints { min: graphics::Vector2f::new(0.0, 0.0), max: window.size().as_other() };

        render_object.layout(size_constraints);

        let mouse_position = window.mouse_position().as_other();
        let hover = render_object.find_hover(view_top_left, mouse_position);

        window.clear(graphics::Color::BLACK);
        render_object.draw(&mut window, view_top_left, hover);

        window.display();
    }
}
