use crate::{
    error::{Error, ErrorReportedPromise, Report},
    lang::{BinaryOp, Expr, UnaryOp, VarName, ShortCircuitOp},
    parser::{parser::Parser, token::Token, SyntaxOptions},
};

pub(super) fn expression(parser: &mut Parser, syntax_options: SyntaxOptions) -> Result<Expr, ErrorReportedPromise> {
    or(parser, syntax_options)
}

macro_rules! left_associative_binary_op {
    ($name:ident, $next_level:ident, $expr_variant:ident, $operator_predicate:expr $(,)?) => {
        fn $name(parser: &mut Parser, syntax_options: SyntaxOptions) -> Result<Expr, ErrorReportedPromise> {
            let mut left = $next_level(parser, syntax_options)?;

            while let Some(op) = parser.maybe_consume($operator_predicate) {
                let right = $next_level(parser, syntax_options)?;
                left = Expr::$expr_variant(Box::new(left), op, Box::new(right));
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

fn unary(parser: &mut Parser, syntax_options: SyntaxOptions) -> Result<Expr, ErrorReportedPromise> {
    if let Some(operator) = parser.maybe_consume(|tok| match tok.1 {
        Token::Bang => Some(UnaryOp::LogicalNegate),
        Token::Minus => Some(UnaryOp::NumericNegate),
        _ => None,
    }) {
        let operand = unary(parser, syntax_options)?;
        Ok(Expr::UnaryOp(operator, Box::new(operand)))
    } else {
        call(parser, syntax_options)
    }
}

fn call(parser: &mut Parser, syntax_options: SyntaxOptions) -> Result<Expr, ErrorReportedPromise> {
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

        parser.consume(|tok| match tok.1 {
            Token::CParen => Ok(()),
            _ => Err({ Error::new(Some(tok.0), "expected ')' after arguments".to_string()) }.report()),
        })?;

        expr = Expr::Call(Box::new(expr), arguments);
    }

    Ok(expr)
}

fn primary(parser: &mut Parser, syntax_options: SyntaxOptions) -> Result<Expr, ErrorReportedPromise> {
    let next = parser.next();
    match next.1 {
        Token::Identifier(n) => Ok(Expr::Var(VarName(n))),
        Token::IntLit(i) => Ok(Expr::Int(i)),
        Token::FloatLit(f) => Ok(Expr::Float(f)),
        Token::StrLit(s) => Ok(Expr::String(s)),
        Token::BoolLit(b) => Ok(Expr::Bool(b)),

        Token::OParen => Ok(Expr::Parenthesized(Box::new(expression(parser, syntax_options)?))),

        _ => Err(Error::new(Some(next.0), "expected expression".to_string()).report()),
    }
}
