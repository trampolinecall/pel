use num_bigint::BigInt;

#[derive(Clone)]
pub(super) enum Token {
    OParen,
    CParen,
    OBrack,
    CBrack,
    OBrace,
    CBrace,

    Semicolon,

    Period,
    Comma,
    Equal,

    Bang,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    Pipe,
    Amper,

    DoublePipe,
    DoubleAmper,

    SlashEqual,
    StarEqual,
    MinusEqual,
    PlusEqual,
    PercentEqual,

    BangEqual,
    DoubleEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier(String),

    IntLit(BigInt),
    FloatLit(f64),
    StrLit(String),
    BoolLit(bool),

    If,
    Else,
    For,
    While,
    Break,
    Continue,
    Var,
    Return,
    Fn,
    Assign,
    To,
    Print,
    Make,

    Eof,
}
