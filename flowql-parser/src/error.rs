use std::ops::Range;

#[derive(Debug)]
pub enum DiagnosticLevel {
    Error,
    Warning,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub title: &'static str,
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}
