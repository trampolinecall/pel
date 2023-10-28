use crate::{
    error::{Error, ErrorReportedPromise, Report},
    io::Located,
    lang::{Expr, Statement, VarName},
    parser::{expr::expression, parser::Parser, token::Token, SyntaxOptions},
};

// TODO: semicolons

pub(super) fn statement(parser: &mut Parser, syntax_options: SyntaxOptions) -> Result<Statement, ErrorReportedPromise> {
    let tok = parser.next();
    match tok.1 {
        Token::OBrace => finish_block(parser, syntax_options, tok),
        Token::If => if_statement(parser, syntax_options, tok),
        Token::For => for_statement(parser, syntax_options, tok),
        Token::While => while_statement(parser, syntax_options, tok),
        Token::Break => break_statement(parser, syntax_options, tok),
        Token::Continue => continue_statement(parser, syntax_options, tok),
        Token::Var => var_statement(parser, syntax_options, tok),
        Token::Return => return_statement(parser, syntax_options, tok),
        Token::Assign => assign_statement(parser, syntax_options, tok),
        Token::Make => make_var_statement(parser, syntax_options, tok),
        Token::Print => print_statement(parser, syntax_options, tok),

        _ => {
            let expr = expression(parser, syntax_options)?;
            if let Some(()) = parser.maybe_consume(|tok| matches!(tok.1, Token::Equal).then_some(())) {
                let rhs = expression(parser, syntax_options)?;
                make_assignment(expr, rhs)
            } else {
                Ok(Statement::Expr(expr))
            }
        }
    }
}

fn finish_block(parser: &mut Parser, syntax_options: SyntaxOptions, obrace_tok: Located<Token>) -> Result<Statement, ErrorReportedPromise> {
    let mut statements = Vec::new();

    while !parser.peek_matches(|tok| matches!(tok, Token::CBrace | Token::Eof)) {
        statements.push(statement(parser, syntax_options)?);
    }

    parser.consume(|tok| match tok.1 {
        Token::CBrace => Ok(tok),
        _ => Err(Error::new(Some(tok.0), "expected '}' to close block".to_string()).report()),
    })?;

    Ok(Statement::Block(statements))
}

fn if_statement(parser: &mut Parser, syntax_options: SyntaxOptions, if_tok: Located<Token>) -> Result<Statement, ErrorReportedPromise> {
    let cond = expression(parser, syntax_options)?;

    let obrace = parser.consume(|tok| match tok.1 {
        Token::OBrace => Ok(tok),
        _ => Err(Error::new(Some(tok.0), "expected '{' after condition of 'if' statement".to_string()).report()),
    })?;

    let true_branch = finish_block(parser, syntax_options, obrace)?;

    let false_branch = if let Some(()) = parser.maybe_consume(|tok| matches!(tok.1, Token::Else).then_some(())) {
        if let Some(if_tok) = parser.maybe_consume(|tok| match tok.1 {
            Token::If => Some(tok),
            _ => None,
        }) {
            Some(if_statement(parser, syntax_options, if_tok)?)
        } else {
            let obrace = parser.consume(|tok| match tok.1 {
                Token::OBrace => Ok(tok),
                _ => Err(Error::new(Some(tok.0), "expected either 'if' or '{' after 'else'".to_string()).report()),
            })?;

            Some(finish_block(parser, syntax_options, obrace)?)
        }
    } else {
        None
    };

    Ok(Statement::If(cond, Box::new(true_branch), false_branch.map(Box::new)))
}

fn for_statement(parser: &mut Parser, syntax_options: SyntaxOptions, for_tok: Located<Token>) -> Result<Statement, ErrorReportedPromise> {
    // TODO: decide about these
    todo!()
}

fn while_statement(parser: &mut Parser, syntax_options: SyntaxOptions, while_tok: Located<Token>) -> Result<Statement, ErrorReportedPromise> {
    let cond = expression(parser, syntax_options)?;

    let obrace = parser.consume(|tok| match tok.1 {
        Token::OBrace => Ok(tok),
        _ => Err(Error::new(Some(tok.0), "expected '{' after condition of 'while' loop".to_string()).report()),
    })?;

    let body = finish_block(parser, syntax_options, obrace)?;

    Ok(Statement::While(cond, Box::new(body)))
}

fn break_statement(parser: &mut Parser, syntax_options: SyntaxOptions, break_tok: Located<Token>) -> Result<Statement, ErrorReportedPromise> {
    todo!()
}

fn continue_statement(parser: &mut Parser, syntax_options: SyntaxOptions, continue_tok: Located<Token>) -> Result<Statement, ErrorReportedPromise> {
    todo!()
}

fn var_statement(parser: &mut Parser, syntax_options: SyntaxOptions, var_tok: Located<Token>) -> Result<Statement, ErrorReportedPromise> {
    let name = parser.consume(|tok| match tok.1 {
        Token::Identifier(name) => Ok(name),
        _ => Err(Error::new(Some(tok.0), "expected variable name after 'var'".to_string()).report()),
    })?;

    let rhs = if let Some(()) = parser.maybe_consume(|tok| matches!(tok.1, Token::Equal).then_some(())) { Some(expression(parser, syntax_options)?) } else { None };

    parser.consume(|tok| match tok.1 {
        Token::Semicolon => Ok(()),
        _ => Err(Error::new(Some(tok.0), "expected ';' after 'var' statement".to_string()).report()),
    })?;

    Ok(Statement::MakeVar(VarName(name), rhs))
}

fn return_statement(parser: &mut Parser, syntax_options: SyntaxOptions, peek: Located<Token>) -> Result<Statement, ErrorReportedPromise> {
    let expr = expression(parser, syntax_options)?;

    parser.consume(|tok| match tok.1 {
        Token::Semicolon => Ok(()),
        _ => Err(Error::new(Some(tok.0), "expected ';' after 'return' statement".to_string()).report()),
    })?;

    Ok(Statement::Return(expr))
}

fn print_statement(parser: &mut Parser, syntax_options: SyntaxOptions, print_tok: Located<Token>) -> Result<Statement, ErrorReportedPromise> {
    let expr = expression(parser, syntax_options)?;

    parser.consume(|tok| match tok.1 {
        Token::Semicolon => Ok(()),
        _ => Err(Error::new(Some(tok.0), "expected ';' after 'print' statement".to_string()).report()),
    })?;

    Ok(Statement::Print(expr))
}

fn assign_statement(parser: &mut Parser, syntax_options: SyntaxOptions, assign_tok: Located<Token>) -> Result<Statement, ErrorReportedPromise> {
    let value = expression(parser, syntax_options)?;

    parser.consume(|tok| match tok.1 {
        Token::To => Ok(()),
        _ => Err(Error::new(Some(tok.0), "expected 'to'".to_string()).report()),
    })?;

    let target = expression(parser, syntax_options)?;

    parser.consume(|tok| match tok.1 {
        Token::Semicolon => Ok(()),
        _ => Err(Error::new(Some(tok.0), "expected ';' after 'assign' statement".to_string()).report()),
    })?;

    make_assignment(target, value)
}

fn make_var_statement(parser: &mut Parser, syntax_options: SyntaxOptions, make_tok: Located<Token>) -> Result<Statement, ErrorReportedPromise> {
    parser.consume(|tok| match tok.1 {
        Token::Var => Ok(()),
        _ => Err(Error::new(Some(tok.0), "expected 'var' after 'make'".to_string()).report()),
    })?;

    let name = parser.consume(|tok| match tok.1 {
        Token::Identifier(name) => Ok(name),
        _ => Err(Error::new(Some(tok.0), "expected variable name after 'var'".to_string()).report()),
    })?;

    parser.consume(|tok| match tok.1 {
        Token::Semicolon => Ok(()),
        _ => Err(Error::new(Some(tok.0), "expected ';' after 'make var' statement".to_string()).report()),
    })?;

    Ok(Statement::MakeVar(VarName(name), None))
}

fn make_assignment(target: Expr, value: Expr) -> Result<Statement, ErrorReportedPromise> {
    match target {
        Expr::Var(vn) => Ok(Statement::AssignVar(vn, value)),
        _ => Err(Error::new(Some(todo!("expr.span")), "invalid assignment target".to_string()).report()),
    }
}
