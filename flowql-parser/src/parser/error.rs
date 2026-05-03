use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxError {
    pub kind: SyntaxErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxErrorKind {
    InternalError(&'static str),
}
