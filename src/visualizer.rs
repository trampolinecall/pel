pub(crate) mod graphics;
pub(crate) mod layout;
pub(crate) mod lens;
pub(crate) mod render_object;
#[macro_use]
pub(crate) mod widgets;
pub(crate) mod vdom;

use crate::visualizer::{
    graphics::{Fonts, GraphicsContext},
    widgets::Widget,
};

pub(crate) fn run<Model, ModelAsWidget: Widget<Model>>(mut model: Model, model_to_widget: impl Fn(&Model) -> ModelAsWidget) {
    let graphics_context = {
        // TODO: REMOVE
        let fonts = {
            let text_font = graphics::Font::Text;

            let monospace_font = graphics::Font::Monospace;

            Fonts { text_font, monospace_font }
        };

        GraphicsContext { fonts }
    };

    let app_div = web_sys::window().expect("no global window").document().expect("no document on window").get_element_by_id("app").expect("no div with id 'app'");

    // TODO: figure out how this is actually supposed to work because it is not supposed to be in an infinite "event loop" (present once, only present again when callbacks run?)
    loop {
        model_to_widget(&model).to_vdom().update_actual_dom(app_div);
        // TODO: vdom.draw(&graphics_context, &mut window, view_top_left, hover);
    }
}
