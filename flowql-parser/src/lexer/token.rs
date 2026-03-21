use crate::error::Span;

pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
}

pub enum TokenKind<'a> {
    // end of file
    Eof,

    // identifier
    Ident(&'a str),

    // literals
    IntLit(i64),
    FloatLit(f64),
    StringLit(&'a str),

    // punctuation
    Semicolon,
    Colon,
    DoubleColon,
    Comma,
    Dot,
    TripleDot,

    // delimiters
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    // operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    LogicalOr,
    LogicalAnd,
    Pipeline,

    // comparisons
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Assign,

    // keywords
    If,
    Let,
    Store,
    Drop,
}
