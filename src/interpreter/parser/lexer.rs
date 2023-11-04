use std::{iter::Peekable, str::CharIndices};

use crate::{
    error::{Error, ErrorReportedPromise, Report},
    interpreter::parser::token::Token,
    source::{File, Located, Span},
};

enum LexError<'file> {
    UnterminatedString(Span<'file>),
    BadCharacter(Span<'file>, char),
}

impl<'file> From<LexError<'file>> for Error<'file> {
    fn from(val: LexError<'file>) -> Self {
        match val {
            LexError::UnterminatedString(sp) => Error::new(Some(sp), "unterminated string literal".into()),

            LexError::BadCharacter(sp, ch) if ch.is_ascii() => Error::new(Some(sp), format!("bad character '{ch}'")),
            LexError::BadCharacter(sp, ch) => Error::new(Some(sp), format!("bad non-ascii character '{ch}'")),
        }
    }
}

pub(super) struct Lexer<'file>(&'file File, Peekable<CharIndices<'file>>);
impl<'file> Lexer<'file> {
    pub(super) fn new(file: &'file File) -> Lexer<'file> {
        Lexer(file, file.source.char_indices().peekable())
    }

    fn pos(&mut self) -> Option<usize> {
        self.1.peek().map(|(i, _)| *i)
    }
    fn peek(&mut self) -> Option<char> {
        self.1.peek().map(|(_, a)| *a)
    }

    fn check_peek_matches_and_consume(&mut self, ch: char) -> bool {
        if self.peek() == Some(ch) {
            self.next();
            true
        } else {
            false
        }
    }

    fn slice(&self, start: usize, end: usize) -> &str {
        &self.0.source[start..end]
    }

    fn slice_from(&mut self, start: usize) -> &str {
        if let Some(end) = self.pos() {
            &self.0.source[start..end]
        } else {
            &self.0.source[start..]
        }
    }

    /* (unused function)
    fn span(&self, (start): Ind, (end): Ind) -> Span<'file> {
        Span(self.0, start, end)
    }
    */

    fn span_from(&mut self, start: usize) -> Span<'file> {
        if let Some(end) = self.pos() {
            Span::new_from_start_and_end(self.0, start, end)
        } else {
            Span::new_from_start_and_end(self.0, start, self.0.source.len())
        }
    }

    fn string(&mut self, start: usize) -> Result<Located<'file, Token>, ErrorReportedPromise> {
        while let Some((end, c)) = self.1.next() {
            if c == '\"' {
                let sp = self.span_from(start);
                let tok = Token::StrLit(self.slice(start + 1, end).into());
                return Ok(Located(sp, tok));
            }
        }

        Err(LexError::UnterminatedString(self.span_from(start)).report())
    }

    fn number(&mut self, start: usize) -> Located<'file, Token> {
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.1.next();
        }

        if self.peek() == Some('.') {
            self.1.next();
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                self.1.next();
            }

            let tok = Token::FloatLit(self.slice_from(start).parse().unwrap());
            Located(self.span_from(start), tok)
        } else {
            let tok = Token::IntLit(self.slice_from(start).parse().unwrap());
            Located(self.span_from(start), tok)
        }
    }

    fn alpha_iden(&mut self, start: usize) -> Located<'file, Token> {
        while self.peek().map_or(false, |c| c.is_ascii_alphanumeric() || c == '_') {
            self.1.next();
        }
        Located(
            self.span_from(start),
            match self.slice_from(start) {
                "if" => Token::If,
                "else" => Token::Else,
                "for" => Token::For,
                "while" => Token::While,
                "break" => Token::Break,
                "continue" => Token::Continue,
                "var" => Token::Var,
                "return" => Token::Return,
                "fn" => Token::Fn,
                "assign" => Token::Assign,
                "make" => Token::Make,
                "print" => Token::Print,
                "to" => Token::To,
                "true" => Token::BoolLit(true),
                "false" => Token::BoolLit(false),
                iden => Token::Identifier(iden.into()),
            },
        )
    }

    pub(super) fn next(&mut self) -> Located<'file, Token> {
        let Some((start_ind, c)) = self.1.next() else {
            return Located(self.0.eof_span(), Token::Eof);
        };

        match c {
            '/' if self.peek() == Some('/') => {
                loop {
                    if let Some(&(_, '\n')) = self.1.peek() {
                        break;
                    }
                    self.1.next();
                }

                self.next()
            }

            '"' => match self.string(start_ind) {
                Ok(s) => s,
                Err(_) => self.next(),
            },

            ' ' | '\n' | '\t' => self.next(),

            '(' => Located(self.span_from(start_ind), Token::OParen),
            ')' => Located(self.span_from(start_ind), Token::CParen),
            '[' => Located(self.span_from(start_ind), Token::OBrack),
            ']' => Located(self.span_from(start_ind), Token::CBrack),
            '{' => Located(self.span_from(start_ind), Token::OBrace),
            '}' => Located(self.span_from(start_ind), Token::CBrace),
            ';' => Located(self.span_from(start_ind), Token::Semicolon),
            '.' => Located(self.span_from(start_ind), Token::Period),
            ',' => Located(self.span_from(start_ind), Token::Comma),

            '=' => {
                if self.check_peek_matches_and_consume('=') {
                    Located(self.span_from(start_ind), Token::DoubleEqual)
                } else {
                    Located(self.span_from(start_ind), Token::Equal)
                }
            }
            '!' => {
                if self.check_peek_matches_and_consume('=') {
                    Located(self.span_from(start_ind), Token::BangEqual)
                } else {
                    Located(self.span_from(start_ind), Token::Bang)
                }
            }
            '+' => {
                if self.check_peek_matches_and_consume('=') {
                    Located(self.span_from(start_ind), Token::PlusEqual)
                } else {
                    Located(self.span_from(start_ind), Token::Plus)
                }
            }
            '-' => {
                if self.check_peek_matches_and_consume('=') {
                    Located(self.span_from(start_ind), Token::MinusEqual)
                } else {
                    Located(self.span_from(start_ind), Token::Minus)
                }
            }
            '*' => {
                if self.check_peek_matches_and_consume('=') {
                    Located(self.span_from(start_ind), Token::StarEqual)
                } else {
                    Located(self.span_from(start_ind), Token::Star)
                }
            }
            '/' => {
                if self.check_peek_matches_and_consume('=') {
                    Located(self.span_from(start_ind), Token::SlashEqual)
                } else {
                    Located(self.span_from(start_ind), Token::Slash)
                }
            }
            '%' => {
                if self.check_peek_matches_and_consume('=') {
                    Located(self.span_from(start_ind), Token::PercentEqual)
                } else {
                    Located(self.span_from(start_ind), Token::Percent)
                }
            }

            '|' => {
                if self.check_peek_matches_and_consume('|') {
                    Located(self.span_from(start_ind), Token::DoublePipe)
                } else {
                    Located(self.span_from(start_ind), Token::Pipe)
                }
            }
            '&' => {
                if self.check_peek_matches_and_consume('&') {
                    Located(self.span_from(start_ind), Token::DoubleAmper)
                } else {
                    Located(self.span_from(start_ind), Token::Amper)
                }
            }

            '>' => {
                if self.check_peek_matches_and_consume('=') {
                    Located(self.span_from(start_ind), Token::GreaterEqual)
                } else {
                    Located(self.span_from(start_ind), Token::Greater)
                }
            }
            '<' => {
                if self.check_peek_matches_and_consume('=') {
                    Located(self.span_from(start_ind), Token::LessEqual)
                } else {
                    Located(self.span_from(start_ind), Token::Less)
                }
            }

            c if c.is_ascii_digit() => self.number(start_ind),
            c if c.is_ascii_alphabetic() || c == '_' => self.alpha_iden(start_ind),

            _ => {
                LexError::BadCharacter(self.span_from(start_ind), c).report();
                self.next()
            }
        }
    }
}
