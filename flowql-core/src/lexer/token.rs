use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords — reserved words
    Let,
    Table,
    If,
    Else,
    True,
    False,

    // Identifiers & Literals
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    Instant(i64, u32),
    Duration(i64, u32),
    Now,
    Today,
    TodayAt(u32, u32, u32, u32),

    // Arithmetic / Concatenation
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    PlusPlus,

    // Comparison
    EqEq,
    BangEq,
    LAngleEq,
    RAngleEq,

    // Logical
    Bang,
    AmpAmp,
    PipePipe,

    // Range
    DotDot,
    DotDotEq,

    // Pipeline & Access
    PipeR,
    Dot,
    Colon,
    ColonColon,
    At,
    Question,

    // Assignment
    Eq,

    // Delimiters & Separators
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    LAngle,
    RAngle,
    Semicolon,
    Comma,

    // Special
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Keywords
            TokenKind::Let => write!(f, "let"),
            TokenKind::Table => write!(f, "table"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            // Identifiers & Literals
            TokenKind::Ident(s) => write!(f, "{}", s),
            TokenKind::Int(n) => write!(f, "{}", n),
            TokenKind::Float(n) => write!(f, "{}", n),
            TokenKind::Str(s) => write!(f, "\"{}\"", s),
            TokenKind::Instant(secs, nanos) => write!(f, "@unix_{}.{:09}", secs, nanos),
            TokenKind::Duration(secs, nanos) => write!(f, "#{}.{:09}s", secs, nanos),
            TokenKind::Now => write!(f, "@now"),
            TokenKind::Today => write!(f, "@today"),
            TokenKind::TodayAt(h, m, s, n) => {
                if *n == 0 {
                    write!(f, "@today_{:02}:{:02}:{:02}", h, m, s)
                } else {
                    write!(f, "@today_{:02}:{:02}:{:02}.{:09}", h, m, s, n)
                }
            }
            // Arithmetic / Concatenation
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::PlusPlus => write!(f, "++"),
            // Comparison
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::BangEq => write!(f, "!="),
            TokenKind::LAngleEq => write!(f, "<="),
            TokenKind::RAngleEq => write!(f, ">="),
            // Logical
            TokenKind::Bang => write!(f, "!"),
            TokenKind::AmpAmp => write!(f, "&&"),
            TokenKind::PipePipe => write!(f, "||"),
            // Range
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::DotDotEq => write!(f, "..="),
            // Pipeline & Access
            TokenKind::PipeR => write!(f, "|>"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::ColonColon => write!(f, "::"),
            TokenKind::At => write!(f, "@"),
            TokenKind::Question => write!(f, "?"),
            // Assignment
            TokenKind::Eq => write!(f, "="),
            // Delimiters & Separators
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LAngle => write!(f, "<"),
            TokenKind::RAngle => write!(f, ">"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Comma => write!(f, ","),
            // Special
            TokenKind::Eof => write!(f, "<eof>"),
        }
    }
}
