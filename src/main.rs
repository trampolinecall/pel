#![warn(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::type_complexity)]

mod source;

mod error;

#[macro_use]
mod visualizer;

mod interpreter {
    #[allow(clippy::module_inception)]
    pub(crate) mod interpreter;
    pub(crate) mod lang;
    pub(crate) mod parser;
}

use std::process::ExitCode;

use crate::error::Report;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn run() -> Result<(), error::ErrorReportedPromise> {
    let file = {
        let mut args = std::env::args();
        if args.len() != 2 {
            return Err(error::Error::new(None, "expected 1 argument: input file".to_string()).report());
        } else {
            let name = args.nth(1).expect("args should have 2 items because that is checked in the if clause above");
            let source = std::fs::read_to_string(&name).map_err(|err| error::Error::new(None, format!("error opening file: {err}")).report())?;
            source::File::new(name, source)
        }
    };

    let syntax_options =
        interpreter::parser::SyntaxOptions { assign_type: interpreter::parser::AssignStatementType::Keyword, variable_decl_type: interpreter::parser::VariableDeclarationType::Keyword };
    let stmts = interpreter::parser::parse_statements(&file, syntax_options)?;

    let interpreter = interpreter::interpreter::new_interpreter(stmts);
    visualizer::run("pel interpreter", (800, 600), interpreter, interpreter::interpreter::InterpreterViewer::view);

    Ok(())
}
