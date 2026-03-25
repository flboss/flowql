use crate::error::{Diagnostic, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct LexerError {
    pub(crate) kind: LexerErrorKind,
    pub(crate) span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LexerErrorKind {
    UnknownCharacter(char),
    IncompleteToken(&'static str),
    UnclosedComment,
    UnclosedStringLit,
    InvalidEscapeSequence,
    IntegerOverflow,
    HexadecimalFloat,
    OctalFloat,
    BinaryFloat,
    IncompleteFloatExponent,
    InternalError(&'static str),
}

impl LexerError {
    pub fn to_diagnostic(self) -> Diagnostic {
        todo!()
    }
}
