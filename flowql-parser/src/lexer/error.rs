use crate::error::{Diagnostic, Span};

pub struct LexerError {
    pub(crate) kind: LexerErrorKind,
    pub(crate) span: Span,
}

// TODO: add info to error variants
pub(crate) enum LexerErrorKind {
    UnknownCharacter(char),
    IncompleteToken(&'static str),
    UnclosedComment,
    UnclosedStringLit,
    MalformedFloat, // TODO: split into several errors?
    InternalError(&'static str),
}

impl LexerError {
    pub fn to_diagnostic(self) -> Diagnostic {
        todo!()
    }
}
