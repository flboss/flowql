use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct LexicalError {
    pub kind: LexicalErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexicalErrorKind {
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
