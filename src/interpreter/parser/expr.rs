use crate::{
    error::{Error, ErrorReportedPromise, Report},
    source::Located,
    interpreter::lang::{BinaryOp, Expr, ExprKind, ShortCircuitOp, UnaryOp, VarName},
    interpreter::parser::{parser::Parser, token::Token, SyntaxOptions},
};

pub(super) fn expression<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions) -> Result<Expr<'file>, ErrorReportedPromise> {
    or(parser, syntax_options)
}

macro_rules! left_associative_binary_op {
    ($name:ident, $next_level:ident, $expr_variant:ident, $operator_predicate:expr $(,)?) => {
        fn $name<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions) -> Result<Expr<'file>, ErrorReportedPromise> {
            let mut left = $next_level(parser, syntax_options)?;

            while let Some(op) = parser.maybe_consume($operator_predicate) {
                let right = $next_level(parser, syntax_options)?;
                let span = left.span + right.span;
                left = Expr { kind: ExprKind::$expr_variant(Box::new(left), op, Box::new(right)), span };
            }

            Ok(left)
        }
    };
}

left_associative_binary_op!(or, and, ShortCircuitOp, |tok| match tok.1 {
    Token::DoublePipe => Some(ShortCircuitOp::Or),
    _ => None,
});
left_associative_binary_op!(and, equality, ShortCircuitOp, |tok| match tok.1 {
    Token::DoubleAmper => Some(ShortCircuitOp::And),
    _ => None,
});
left_associative_binary_op!(equality, comparison, BinaryOp, |tok| match tok.1 {
    Token::BangEqual => Some(BinaryOp::NotEqual),
    Token::DoubleEqual => Some(BinaryOp::Equal),
    _ => None,
});
left_associative_binary_op!(comparison, term, BinaryOp, |tok| match tok.1 {
    Token::Greater => Some(BinaryOp::Greater),
    Token::GreaterEqual => Some(BinaryOp::GreaterEqual),
    Token::Less => Some(BinaryOp::Less),
    Token::LessEqual => Some(BinaryOp::LessEqual),
    _ => None,
});
left_associative_binary_op!(term, factor, BinaryOp, |tok| match tok.1 {
    Token::Plus => Some(BinaryOp::Add),
    Token::Minus => Some(BinaryOp::Subtract),
    _ => None,
});
left_associative_binary_op!(factor, unary, BinaryOp, |tok| match tok.1 {
    Token::Star => Some(BinaryOp::Multiply),
    Token::Slash => Some(BinaryOp::Divide),
    Token::Percent => Some(BinaryOp::Modulo),
    _ => None,
});

fn unary<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions) -> Result<Expr<'file>, ErrorReportedPromise> {
    if let Some(operator) = parser.maybe_consume(|tok| match tok.1 {
        Token::Bang => Some(Located(tok.0, UnaryOp::LogicalNegate)),
        Token::Minus => Some(Located(tok.0, UnaryOp::NumericNegate)),
        _ => None,
    }) {
        let operand = unary(parser, syntax_options)?;
        let total_span = operator.0 + operand.span;
        Ok(Expr { kind: ExprKind::UnaryOp(operator.1, Box::new(operand)), span: total_span })
    } else {
        call(parser, syntax_options)
    }
}

fn call<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions) -> Result<Expr<'file>, ErrorReportedPromise> {
    let mut expr = primary(parser, syntax_options)?;

    while let Some(()) = parser.maybe_consume(|tok| match tok.1 {
        Token::OParen => Some(()),
        _ => None,
    }) {
        let mut arguments = Vec::new();
        if !parser.peek_matches(|tok| matches!(tok, Token::CParen)) {
            arguments.push(expression(parser, syntax_options)?);
            while let Some(()) = parser.maybe_consume(|tok| match tok.1 {
                Token::Comma => Some(()),
                _ => None,
            }) {
                arguments.push(expression(parser, syntax_options)?);
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

fn primary<'file>(parser: &mut Parser<'file>, syntax_options: SyntaxOptions) -> Result<Expr<'file>, ErrorReportedPromise> {
    let next = parser.next();
    match next.1 {
        Token::Identifier(n) => Ok(Expr { kind: ExprKind::Var(VarName(n)), span: next.0 }),
        Token::IntLit(i) => Ok(Expr { kind: ExprKind::Int(i), span: next.0 }),
        Token::FloatLit(f) => Ok(Expr { kind: ExprKind::Float(f), span: next.0 }),
        Token::StrLit(s) => Ok(Expr { kind: ExprKind::String(s), span: next.0 }),
        Token::BoolLit(b) => Ok(Expr { kind: ExprKind::Bool(b), span: next.0 }),

        Token::OParen => {
            let inner = expression(parser, syntax_options)?;

            let cparen_sp = parser.consume(|tok| match tok.1 {
                Token::CParen => Ok(tok.0),
                _ => Err({ Error::new(Some(tok.0), "expected ')' to close parenthesized expression".to_string()) }.report()),
            })?;

            Ok(Expr { kind: ExprKind::Parenthesized(Box::new(inner)), span: next.0 + cparen_sp })
        }

        _ => Err(Error::new(Some(next.0), "expected expression".to_string()).report()),
    }
}
