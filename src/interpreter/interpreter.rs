mod interpreter;

use std::future::Future;

use genawaiter::sync::Gen;

use crate::{
    interpreter::{
        interpreter::interpreter::{value::ReprValue, InterpretYield, RuntimeError},
        lang::Stmt,
    },
    visualizer::{
        graphics::Fonts,
        widgets::{code_view::code_view, either::Either, flex, label::Label, responds_to_keyboard::RespondsToKeyboard, Widget},
    },
};

pub(crate) struct InterpreterViewer<'file, F: Future<Output = Result<(), RuntimeError<'file>>>> {
    state: InterpreterViewState<'file>,

    generator: Gen<InterpretYield<'file>, (), F>,
}
enum InterpreterViewState<'file> {
    NotStarted,
    AboutToExecute(InterpretYield<'file>),
    Finished { result: Result<(), RuntimeError<'file>> },
}

pub(crate) fn new_interpreter(stmts: Vec<Stmt>) -> InterpreterViewer<impl Future<Output = Result<(), RuntimeError>>> {
    let gen = Gen::new(move |co| interpreter::interpret(stmts, co));
    InterpreterViewer { state: InterpreterViewState::NotStarted, generator: gen }
}
impl<'file, F: Future<Output = Result<(), RuntimeError<'file>>> + 'file> InterpreterViewer<'file, F> {
    pub(crate) fn view(&self) -> impl Widget<InterpreterViewer<'file, F>> {
        let make_message = |message| Either::new_left(Label::new(message, Fonts::text_font, 15));
        let widget = match &self.state {
            InterpreterViewState::NotStarted => make_message("interpreter not started".to_string()),
            InterpreterViewState::AboutToExecute(InterpretYield { msg, highlight, state }) => {
                // TODO: hashmap does not preserve order that variables are created
                // TODO: var and value side by side in table aligned
                let env_view = flex::homogeneous::Flex::new(
                    flex::Direction::Vertical,
                    state
                        .env
                        .scopes
                        .iter()
                        .flat_map(|env_scope| {
                            env_scope.iter().map(|(var_name, value)| {
                                (
                                    flex::ItemSettings::Fixed,
                                    Label::new(
                                        match value {
                                            // TODO: min height?
                                            Some(value) => format!("{var_name}: {}", ReprValue(value)),
                                            None => format!("{var_name}: <uninitialized>"),
                                        },
                                        Fonts::text_font,
                                        15,
                                    ),
                                )
                            })
                        })
                        .collect(),
                );

                Either::new_right(flex! {
                    horizontal
                    code_view: ItemSettings::Flex(0.8), code_view(*highlight, Fonts::text_font, 15, Fonts::monospace_font, 15),
                    program_output: ItemSettings::Flex(0.3), Label::new(state.program_output.clone(), Fonts::monospace_font, 15), // TODO: scrolling, min size, fixed size?, scroll to bottom automatically
                    env_view: ItemSettings::Flex(0.2), env_view,
                    msg: ItemSettings::Flex(0.2), Label::new(format!("running\n{msg}"), Fonts::text_font, 15),
                })
            }
            InterpreterViewState::Finished { result: Ok(()) } => make_message("interpreter finished successfully".to_string()),
            InterpreterViewState::Finished { result: Err(err) } => make_message(format!("interpreter had error: {err}")),
        };

        RespondsToKeyboard::<Self, _, _>::new(sfml::window::Key::Space, |interpreter: &mut _| interpreter.step(), widget)
    }

    fn step(&mut self) {
        match self.state {
            InterpreterViewState::NotStarted | InterpreterViewState::AboutToExecute { .. } => match self.generator.resume() {
                genawaiter::GeneratorState::Yielded(step) => self.state = InterpreterViewState::AboutToExecute(step),
                genawaiter::GeneratorState::Complete(res) => self.state = InterpreterViewState::Finished { result: res },
            },

            InterpreterViewState::Finished { result: _, .. } => {}
        }
    }
}
