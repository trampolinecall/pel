use crate::{
    error::{Error, ErrorReportedPromise, Report},
    source::File,
    interpreter::lang::{Expr, Stmt},
    interpreter::parser::token::Token,
};

// TODO: syntax options
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum AssignStatementType {
    Keyword,
    Symbol,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum VariableDeclarationType {
    Keyword,
    Symbol,
}
#[derive(Copy, Clone)]
pub(crate) struct SyntaxOptions {
    pub(crate) assign_type: AssignStatementType,
    pub(crate) variable_decl_type: VariableDeclarationType,
}

mod token;

mod lexer;
#[allow(clippy::module_inception)]
mod parser;

mod expr;
mod stmt;

pub(crate) fn parse_expr(file: &File, syntax_options: SyntaxOptions) -> Result<Expr, ErrorReportedPromise> {
    let mut parser = parser::Parser::new(lexer::Lexer::new(file));
    let expr = expr::expression(&mut parser, syntax_options);
    parser.consume(|tok| match tok.1 {
        Token::Eof => Ok(()),
        _ => Err(Error::new(Some(tok.0), "extraneous input".to_string()).report()),
    })?;
    expr
}

pub(crate) fn parse_statements(file: &File, syntax_options: SyntaxOptions) -> Result<Vec<Stmt>, ErrorReportedPromise> {
    let mut parser = parser::Parser::new(lexer::Lexer::new(file));
    let mut statements = Vec::new();
    while !parser.peek_matches(|tok| matches!(tok, Token::Eof)) {
        statements.push(stmt::statement(&mut parser, syntax_options)?);
        // TODO: panic mode error recovery?
    }

    Ok(statements)
}
