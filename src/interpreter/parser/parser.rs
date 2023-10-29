use crate::{
    error::ErrorReportedPromise,
    source::{Located, Span},
    interpreter::parser::{lexer::Lexer, token::Token},
};

pub(super) struct Parser<'file> {
    lexer: Lexer<'file>,
    peek: Option<Located<'file, Token>>,
}

impl<'file> Parser<'file> {
    pub(super) fn new(lexer: Lexer) -> Parser {
        Parser { lexer, peek: None }
    }

    fn fill_peek(&mut self) {
        if self.peek.is_none() {
            self.peek = Some(self.lexer.next());
        }
    }

    pub(super) fn next(&mut self) -> Located<'file, Token> {
        match self.peek.take() {
            Some(tok) => tok,
            None => self.lexer.next(),
        }
    }

    pub(super) fn peek(&mut self) -> &Located<'file, Token> {
        self.fill_peek();
        self.peek.as_ref().expect("peek should not be None because it was just filled")
    }

    pub(super) fn peek_matches(&mut self, pred: impl Fn(&Token) -> bool) -> bool {
        pred(&self.peek().1)
    }

    pub(super) fn consume<R>(&mut self, f: impl Fn(Located<'file, Token>) -> Result<R, ErrorReportedPromise>) -> Result<R, ErrorReportedPromise> {
        self.fill_peek();
        f(self.peek.take().expect("peek should not be None because it was just filled"))
    }

    pub(super) fn maybe_consume<R>(&mut self, f: impl Fn(Located<'file, Token>) -> Option<R>) -> Option<R> {
        self.fill_peek();
        let result = f(self.peek.as_ref().expect("peek should not be None because it was just filled").clone());
        if let Some(result) = result {
            self.peek = None;
            Some(result)
        } else {
            None
        }
    }

    pub(crate) fn peek_span(&mut self) -> Span<'file> {
        self.peek().0
    }
}
