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

use wasm_bindgen::prelude::*;

use crate::error::Report;

#[wasm_bindgen(start)]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    match run() {
        Ok(()) => (),
        Err(_) => panic!("error"), // TODO: do this better
    }
}

fn run() -> Result<(), error::ErrorReportedPromise> {
    let file = {
        /* TODO: do this properly
        if args.len() != 2 {
            return Err(error::Error::new(None, "expected 1 argument: input file".to_string()).report());
        } else {
            let name = args.nth(1).expect("args should have 2 items because that is checked in the if clause above");
            let source = std::fs::read_to_string(&name).map_err(|err| error::Error::new(None, format!("error opening file: {err}")).report())?;
            source::File::new(name, source)
        }
        */
        let name = "scratch".to_string();
        let source = r#"make var iter;

iter = 0;

while iter < 100 {
    var output;
    var fizz = "";
    var buzz = "";

    if iter % 3 == 0 {
        fizz = "Fizz";
    }
    if iter % 5 == 0 {
        buzz = "Buzz";
    }

    output = fizz + buzz;
    if output != "" {
        print output;
    } else {
        print iter;
    }

    if iter == 15 {
        var x;
        x;
    }

    iter = iter + 1;
}"#
        .to_string();
        source::File::new(name, source)
    };

    let syntax_options =
        interpreter::parser::SyntaxOptions { assign_type: interpreter::parser::AssignStatementType::Keyword, variable_decl_type: interpreter::parser::VariableDeclarationType::Keyword };
    let stmts = interpreter::parser::parse_statements(&file, syntax_options)?;

    let interpreter = interpreter::interpreter::new_interpreter(stmts);
    visualizer::run(interpreter, interpreter::interpreter::InterpreterViewer::view);

    Ok(())
}
