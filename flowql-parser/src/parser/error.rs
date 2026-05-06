use crate::error::Span;
use crate::lexer::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxError {
    pub kind: SyntaxErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxErrorKind {
    UnexpectedToken {
        found: TokenKind<'static>,
    },
    ExpectedToken {
        expected: TokenKind<'static>,
        found: TokenKind<'static>,
    },
    UnclosedDelimiter(char),
    InternalError(&'static str),
}
