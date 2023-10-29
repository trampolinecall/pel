use crate::{
    error::{Error, ErrorReportedPromise, Report},
    interpreter::lang::{Expr, ExprKind, Stmt, StmtKind, VarName},
    interpreter::parser::{expr::expression, parser::Parser, token::Token, SyntaxOptions},
    source::{Located, Span},
};

pub(super) fn statement<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions) -> Result<Stmt<'file>, ErrorReportedPromise> {
    let tok = parser.peek();
    match tok.1 {
        Token::OBrace => {
            let tok = parser.next();
            finish_block(parser, syntax_options, tok)
        }
        Token::If => {
            let tok = parser.next();
            if_statement(parser, syntax_options, tok)
        }
        Token::For => {
            let tok = parser.next();
            for_statement(parser, syntax_options, tok)
        }
        Token::While => {
            let tok = parser.next();
            while_statement(parser, syntax_options, tok)
        }
        Token::Break => {
            let tok = parser.next();
            break_statement(parser, syntax_options, tok)
        }
        Token::Continue => {
            let tok = parser.next();
            continue_statement(parser, syntax_options, tok)
        }
        Token::Var => {
            let tok = parser.next();
            var_statement(parser, syntax_options, tok)
        }
        Token::Return => {
            let tok = parser.next();
            return_statement(parser, syntax_options, tok)
        }
        Token::Assign => {
            let tok = parser.next();
            assign_statement(parser, syntax_options, tok)
        }
        Token::Make => {
            let tok = parser.next();
            make_var_statement(parser, syntax_options, tok)
        }
        Token::Print => {
            let tok = parser.next();
            print_statement(parser, syntax_options, tok)
        }

        _ => {
            let expr = expression(parser, syntax_options)?;

            if let Some(()) = parser.maybe_consume(|tok| matches!(tok.1, Token::Equal).then_some(())) {
                let rhs = expression(parser, syntax_options)?;
                let semi_sp = parser.consume(|tok| match tok.1 {
                    Token::Semicolon => Ok(tok.0),
                    _ => Err(Error::new(Some(tok.0), "expected ';' after assignment statement".to_string()).report()),
                })?;
                let total_span = expr.span + semi_sp;
                make_assignment(expr, rhs, total_span)
            } else {
                let expr_span = expr.span;
                let semi_sp = parser.consume(|tok| match tok.1 {
                    Token::Semicolon => Ok(tok.0),
                    _ => Err(Error::new(Some(tok.0), "expected ';' after expression statement".to_string()).report()),
                })?;
                Ok(Stmt { kind: StmtKind::Expr(expr), span: expr_span + semi_sp })
            }
        }
    }
}

fn finish_block<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions, obrace_tok: Located<'file, Token>) -> Result<Stmt<'file>, ErrorReportedPromise> {
    let mut statements = Vec::new();

    while !parser.peek_matches(|tok| matches!(tok, Token::CBrace | Token::Eof)) {
        statements.push(statement(parser, syntax_options)?);
    }

    let cbrace_sp = parser.consume(|tok| match tok.1 {
        Token::CBrace => Ok(tok.0),
        _ => Err(Error::new(Some(tok.0), "expected '}' to close block".to_string()).report()),
    })?;

    Ok(Stmt { kind: StmtKind::Block(statements), span: obrace_tok.0 + cbrace_sp })
}

fn if_statement<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions, if_tok: Located<'file, Token>) -> Result<Stmt<'file>, ErrorReportedPromise> {
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

    let total_span = if_tok.0 + false_branch.as_ref().map(|branch| branch.span).unwrap_or(true_branch.span);

    Ok(Stmt { kind: StmtKind::If(cond, Box::new(true_branch), false_branch.map(Box::new)), span: total_span })
}

fn for_statement<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions, for_tok: Located<Token>) -> Result<Stmt<'file>, ErrorReportedPromise> {
    // TODO: decide about these
    todo!()
}

fn while_statement<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions, while_tok: Located<'file, Token>) -> Result<Stmt<'file>, ErrorReportedPromise> {
    let cond = expression(parser, syntax_options)?;

    let obrace = parser.consume(|tok| match tok.1 {
        Token::OBrace => Ok(tok),
        _ => Err(Error::new(Some(tok.0), "expected '{' after condition of 'while' loop".to_string()).report()),
    })?;

    let body = finish_block(parser, syntax_options, obrace)?;

    let total_span = while_tok.0 + body.span;

    Ok(Stmt { kind: StmtKind::While(cond, Box::new(body)), span: total_span })
}

fn break_statement<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions, break_tok: Located<Token>) -> Result<Stmt<'file>, ErrorReportedPromise> {
    todo!()
}

fn continue_statement<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions, continue_tok: Located<Token>) -> Result<Stmt<'file>, ErrorReportedPromise> {
    todo!()
}

fn var_statement<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions, var_tok: Located<'file, Token>) -> Result<Stmt<'file>, ErrorReportedPromise> {
    let name = parser.consume(|tok| match tok.1 {
        Token::Identifier(name) => Ok(name),
        _ => Err(Error::new(Some(tok.0), "expected variable name after 'var'".to_string()).report()),
    })?;

    let rhs = if let Some(()) = parser.maybe_consume(|tok| matches!(tok.1, Token::Equal).then_some(())) { Some(expression(parser, syntax_options)?) } else { None };

    let semi_sp = parser.consume(|tok| match tok.1 {
        Token::Semicolon => Ok(tok.0),
        _ => Err(Error::new(Some(tok.0), "expected ';' after 'var' statement".to_string()).report()),
    })?;

    Ok(Stmt { kind: StmtKind::MakeVar(VarName(name), rhs), span: var_tok.0 + semi_sp })
}

fn return_statement<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions, return_tok: Located<'file, Token>) -> Result<Stmt<'file>, ErrorReportedPromise> {
    let expr = expression(parser, syntax_options)?;

    let semi_sp = parser.consume(|tok| match tok.1 {
        Token::Semicolon => Ok(tok.0),
        _ => Err(Error::new(Some(tok.0), "expected ';' after 'return' statement".to_string()).report()),
    })?;

    Ok(Stmt { kind: StmtKind::Return(expr), span: return_tok.0 + semi_sp })
}

fn print_statement<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions, print_tok: Located<'file, Token>) -> Result<Stmt<'file>, ErrorReportedPromise> {
    let expr = expression(parser, syntax_options)?;

    let semi_sp = parser.consume(|tok| match tok.1 {
        Token::Semicolon => Ok(tok.0),
        _ => Err(Error::new(Some(tok.0), "expected ';' after 'print' statement".to_string()).report()),
    })?;

    Ok(Stmt { kind: StmtKind::Print(expr), span: print_tok.0 + semi_sp })
}

fn assign_statement<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions, assign_tok: Located<'file, Token>) -> Result<Stmt<'file>, ErrorReportedPromise> {
    let value = expression(parser, syntax_options)?;

    parser.consume(|tok| match tok.1 {
        Token::To => Ok(()),
        _ => Err(Error::new(Some(tok.0), "expected 'to'".to_string()).report()),
    })?;

    let target = expression(parser, syntax_options)?;

    let semi_sp = parser.consume(|tok| match tok.1 {
        Token::Semicolon => Ok(tok.0),
        _ => Err(Error::new(Some(tok.0), "expected ';' after 'assign' statement".to_string()).report()),
    })?;

    make_assignment(target, value, assign_tok.0 + semi_sp)
}

fn make_var_statement<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions, make_tok: Located<'file, Token>) -> Result<Stmt<'file>, ErrorReportedPromise> {
    parser.consume(|tok| match tok.1 {
        Token::Var => Ok(()),
        _ => Err(Error::new(Some(tok.0), "expected 'var' after 'make'".to_string()).report()),
    })?;

    let name = parser.consume(|tok| match tok.1 {
        Token::Identifier(name) => Ok(name),
        _ => Err(Error::new(Some(tok.0), "expected variable name after 'var'".to_string()).report()),
    })?;

    let semi_sp = parser.consume(|tok| match tok.1 {
        Token::Semicolon => Ok(tok.0),
        _ => Err(Error::new(Some(tok.0), "expected ';' after 'make var' statement".to_string()).report()),
    })?;

    Ok(Stmt { kind: StmtKind::MakeVar(VarName(name), None), span: make_tok.0 + semi_sp })
}

fn make_assignment<'file>(target: Expr<'file>, value: Expr<'file>, span: Span<'file>) -> Result<Stmt<'file>, ErrorReportedPromise> {
    match target.kind {
        ExprKind::Var(vn) => Ok(Stmt { kind: StmtKind::AssignVar(vn, value), span }),
        _ => Err(Error::new(Some(target.span), "invalid assignment target".to_string()).report()),
    }
}
