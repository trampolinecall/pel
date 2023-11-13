use crate::{
    error::{Error, ErrorReportedPromise, Report},
    interpreter::lang::{BinaryOp, Expr, ExprKind, ShortCircuitOp, UnaryOp, VarName},
    interpreter::parser::{parser::Parser, token::Token},
    source::Located,
};

pub(super) fn expression<'file>(parser: &mut Parser<'file>) -> Result<Expr<'file>, ErrorReportedPromise> {
    or(parser)
}

macro_rules! left_associative_binary_op {
    ($name:ident, $next_level:ident, $expr_variant:ident, $operator_predicate:expr $(,)?) => {
        fn $name<'file>(parser: &mut Parser<'file>) -> Result<Expr<'file>, ErrorReportedPromise> {
            let mut left = $next_level(parser)?;

            while let Some(op) = parser.maybe_consume($operator_predicate) {
                let right = $next_level(parser)?;
                let span = left.span + right.span;
                left = Expr { kind: ExprKind::$expr_variant(Box::new(left), op, Box::new(right)), span };
            }

            Ok(left)
        }
    };
}

left_associative_binary_op!(or, and, ShortCircuitOp, |tok| match tok.1 {
    Token::DoublePipe => Some(Located(tok.0, ShortCircuitOp::Or)),
    _ => None,
});
left_associative_binary_op!(and, equality, ShortCircuitOp, |tok| match tok.1 {
    Token::DoubleAmper => Some(Located(tok.0, ShortCircuitOp::And)),
    _ => None,
});
left_associative_binary_op!(equality, comparison, BinaryOp, |tok| match tok.1 {
    Token::BangEqual => Some(Located(tok.0, BinaryOp::NotEqual)),
    Token::DoubleEqual => Some(Located(tok.0, BinaryOp::Equal)),
    _ => None,
});
left_associative_binary_op!(comparison, term, BinaryOp, |tok| match tok.1 {
    Token::Greater => Some(Located(tok.0, BinaryOp::Greater)),
    Token::GreaterEqual => Some(Located(tok.0, BinaryOp::GreaterEqual)),
    Token::Less => Some(Located(tok.0, BinaryOp::Less)),
    Token::LessEqual => Some(Located(tok.0, BinaryOp::LessEqual)),
    _ => None,
});
left_associative_binary_op!(term, factor, BinaryOp, |tok| match tok.1 {
    Token::Plus => Some(Located(tok.0, BinaryOp::Add)),
    Token::Minus => Some(Located(tok.0, BinaryOp::Subtract)),
    _ => None,
});
left_associative_binary_op!(factor, unary, BinaryOp, |tok| match tok.1 {
    Token::Star => Some(Located(tok.0, BinaryOp::Multiply)),
    Token::Slash => Some(Located(tok.0, BinaryOp::Divide)),
    Token::Percent => Some(Located(tok.0, BinaryOp::Modulo)),
    _ => None,
});

fn unary<'file>(parser: &mut Parser<'file>) -> Result<Expr<'file>, ErrorReportedPromise> {
    if let Some(operator) = parser.maybe_consume(|tok| match tok.1 {
        Token::Bang => Some(Located(tok.0, UnaryOp::LogicalNegate)),
        Token::Minus => Some(Located(tok.0, UnaryOp::NumericNegate)),
        _ => None,
    }) {
        let operand = unary(parser)?;
        let total_span = operator.0 + operand.span;
        Ok(Expr { kind: ExprKind::UnaryOp(operator, Box::new(operand)), span: total_span })
    } else {
        call(parser)
    }
}

fn call<'file>(parser: &mut Parser<'file>) -> Result<Expr<'file>, ErrorReportedPromise> {
    let mut expr = primary(parser)?;

    while let Some(()) = parser.maybe_consume(|tok| match tok.1 {
        Token::OParen => Some(()),
        _ => None,
    }) {
        let mut arguments = Vec::new();
        if !parser.peek_matches(|tok| matches!(tok, Token::CParen)) {
            arguments.push(expression(parser)?);
            while let Some(()) = parser.maybe_consume(|tok| match tok.1 {
                Token::Comma => Some(()),
                _ => None,
            }) {
                arguments.push(expression(parser)?);
            }
        }

        let cparen_sp = parser.consume(|tok| match tok.1 {
            Token::CParen => Ok(tok.0),
            _ => Err({ Error::new(Some(tok.0), "expected ')' after arguments".to_string()) }.report()),
        })?;

        let total_span = expr.span + cparen_sp;

        expr = Expr { kind: ExprKind::Call(Box::new(expr), arguments), span: total_span };
    }

    Ok(expr)
}

fn primary<'file>(parser: &mut Parser<'file>) -> Result<Expr<'file>, ErrorReportedPromise> {
    let next = parser.next();
    match next.1 {
        Token::Identifier(n) => Ok(Expr { kind: ExprKind::Var(VarName(n)), span: next.0 }),
        Token::IntLit(i) => Ok(Expr { kind: ExprKind::Int(i), span: next.0 }),
        Token::FloatLit(f) => Ok(Expr { kind: ExprKind::Float(f), span: next.0 }),
        Token::StrLit(s) => Ok(Expr { kind: ExprKind::String(s), span: next.0 }),
        Token::BoolLit(b) => Ok(Expr { kind: ExprKind::Bool(b), span: next.0 }),

        Token::OParen => {
            let inner = expression(parser)?;

            let cparen_sp = parser.consume(|tok| match tok.1 {
                Token::CParen => Ok(tok.0),
                _ => Err({ Error::new(Some(tok.0), "expected ')' to close parenthesized expression".to_string()) }.report()),
            })?;

            Ok(Expr { kind: ExprKind::Parenthesized(Box::new(inner)), span: next.0 + cparen_sp })
        }

        _ => Err(Error::new(Some(next.0), "expected expression".to_string()).report()),
    }
}
