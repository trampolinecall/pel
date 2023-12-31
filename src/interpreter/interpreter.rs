mod interpreter;

use std::future::Future;

use genawaiter::sync::Gen;

use crate::{
    app::{
        graphics::{self, Color, Key},
        widgets::{code_view::code_view, either::Either, flex, label::Label, min_size::MinSize, padding::Padding, responds_to_keyboard::RespondsToKeyboard, Widget},
    },
    interpreter::{
        interpreter::interpreter::{value::ReprValue, InterpretYield, RuntimeError},
        lang::Stmt,
    },
};

pub(crate) struct Interpreter<'file, F: Future<Output = Result<(), RuntimeError<'file>>>> {
    last_yield: InterpreterViewState<'file>,
    generator: Gen<InterpretYield<'file>, (), F>,
}
enum InterpreterViewState<'file> {
    NotStarted,
    AboutToExecute(InterpretYield<'file>),
    Finished { result: Result<(), RuntimeError<'file>> },
}

pub(crate) fn new_interpreter(stmts: Vec<Stmt>) -> Interpreter<impl Future<Output = Result<(), RuntimeError>>> {
    let gen = Gen::new(move |co| interpreter::interpret(stmts, co));
    Interpreter { last_yield: InterpreterViewState::NotStarted, generator: gen }
}
impl<'file, F: Future<Output = Result<(), RuntimeError<'file>>> + 'file> Interpreter<'file, F> {
    pub(crate) fn view(&self) -> impl Widget<Interpreter<'file, F>> {
        let make_message = |message| Either::new_left(Label::new(message, "sans-serif".to_string(), 15));
        let widget = match &self.last_yield {
            InterpreterViewState::NotStarted => make_message("interpreter not started".to_string()),
            InterpreterViewState::AboutToExecute(InterpretYield { msg, primary_highlight, secondary_highlights, substitutions, state }) => {
                // TODO: hashmap does not preserve order that variables are created
                // TODO: padding constant
                // TODO: adjustable font size

                Either::new_right(Either::new_right(flex!(horizontal {
                    code_view: (
                        flex::ItemSettings::Flex(0.3),
                        Padding::all_around(
                            code_view((*primary_highlight, Color::rgb(50, 100, 50)), secondary_highlights.clone(), substitutions.clone(), "sans-serif".to_string(), 15, "monospace".to_string(), 15),
                            5.0
                        )
                    ), // TODO: pick better colors
                    program_output: (flex::ItemSettings::Flex(0.3), Padding::all_around(Label::new(state.program_output.clone(), "monospace".to_string(), 15), 5.0)), // TODO: scrolling, min size, fixed size?, scroll to bottom automatically
                    env_view: (flex::ItemSettings::Flex(0.2), Padding::all_around(view_env(&state.env), 5.0)),
                    msg: (flex::ItemSettings::Flex(0.2), Padding::all_around(Label::new(format!("running\n{msg}"), "sans-serif".to_string(), 15), 5.0)),
                })))
            }
            InterpreterViewState::Finished { result: Ok(()) } => make_message("interpreter finished successfully".to_string()),
            InterpreterViewState::Finished { result: Err(err) } => Either::new_right(Either::new_left(flex!(horizontal {
                code_view: (
                    flex::ItemSettings::Flex(0.3),
                    Padding::all_around(code_view((err.span, Color::rgb(150, 0, 0)), Vec::new(), Vec::new(), "sans-serif".to_string(), 15, "monospace".to_string(), 15), 5.0)
                ),
                msg: (flex::ItemSettings::Flex(0.3), Padding::all_around(Label::new(format!("interpreter had error: {}", err.kind), "sans-serif".to_string(), 15), 5.0)),
            }))),
        };

        RespondsToKeyboard::<Self, _, _>::new(Key::Space, |interpreter: &mut _| interpreter.step(), widget)
    }

    fn step(&mut self) {
        match self.last_yield {
            InterpreterViewState::NotStarted | InterpreterViewState::AboutToExecute { .. } => match self.generator.resume() {
                genawaiter::GeneratorState::Yielded(step) => self.last_yield = InterpreterViewState::AboutToExecute(step),
                genawaiter::GeneratorState::Complete(res) => self.last_yield = InterpreterViewState::Finished { result: res },
            },

            InterpreterViewState::Finished { result: _, .. } => {}
        }
    }
}

fn view_env<Data>(env: &interpreter::Vars) -> impl Widget<Data> {
    // TODO: var and value side by side in table aligned
    flex::homogeneous::Flex::new(
        flex::Direction::Vertical,
        env.scopes
            .iter()
            .flat_map(|env_scope| {
                env_scope.iter().map(|(var_name, value)| {
                    (
                        // TODO: grid widget
                        flex::ItemSettings::Fixed,
                        MinSize::new(
                            flex!(horizontal {
                                name: (
                                    flex::ItemSettings::Flex(0.5),
                                    Padding::new(MinSize::new(Label::new(var_name.to_string(), "sans-serif".to_string(), 15), graphics::Vector2f::new(50.0, 0.0)), 10.0, 5.0, 10.0, 5.0)
                                ),

                                value: (
                                    flex::ItemSettings::Flex(0.5),
                                    Padding::new(
                                        MinSize::new(
                                            Label::new(
                                                match &value.1 {
                                                    Some(value) => ReprValue(value).to_string(),
                                                    None => "<uninitialized>".to_string(),
                                                },
                                                "sans-serif".to_string(),
                                                15,
                                            ),
                                            graphics::Vector2f::new(50.0, 0.0)
                                        ),
                                        10.0,
                                        5.0,
                                        10.0,
                                        5.0
                                    )
                                ),
                            }),
                            graphics::Vector2f::new(0.0, 25.0),
                        ),
                    )
                })
            })
            .collect(),
    )
}
