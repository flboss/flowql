use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub title: &'static str,
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span() {
        let span1 = Span::new(0, 26);
        let span2 = Span::new(21, 52);
        let span3 = Span::new(1001, 4096);

        assert_eq!(span1.merge(&span2), Span::new(0, 52));
        assert_eq!(span3.merge(&span2), Span::new(21, 4096));
    }
}
