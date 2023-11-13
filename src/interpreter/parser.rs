use crate::{
    error::{Error, ErrorReportedPromise, Report},
    interpreter::lang::{Expr, Stmt},
    interpreter::parser::token::Token,
    source::File,
};

mod token;

mod lexer;
#[allow(clippy::module_inception)]
mod parser;

mod expr;
mod stmt;

pub(crate) fn parse_expr(file: &File) -> Result<Expr, ErrorReportedPromise> {
    let mut parser = parser::Parser::new(lexer::Lexer::new(file));
    let expr = expr::expression(&mut parser);
    parser.consume(|tok| match tok.1 {
        Token::Eof => Ok(()),
        _ => Err(Error::new(Some(tok.0), "extraneous input".to_string()).report()),
    })?;
    expr
}

pub(crate) fn parse_statements(file: &File) -> Result<Vec<Stmt>, ErrorReportedPromise> {
    let mut parser = parser::Parser::new(lexer::Lexer::new(file));
    let mut statements = Vec::new();
    while !parser.peek_matches(|tok| matches!(tok, Token::Eof)) {
        statements.push(stmt::statement(&mut parser)?);
        // TODO: panic mode error recovery?
    }

    Ok(statements)
}
