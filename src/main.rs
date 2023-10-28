mod io;

mod error;

mod interpreter;
mod lang;
mod parser;

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
            io::File { name, source }
        }
    };

    let parse_options = parser::SyntaxOptions { assign_type: parser::AssignStatementType::Keyword, variable_decl_type: parser::VariableDeclarationType::Keyword };

    let parsed = parser::parse_statements(&file, parse_options)?;
    // interpreter::interpret(&parsed)
    //
    todo!()
}
