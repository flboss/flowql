use std::fmt::Display;

use miette::{Diagnostic, LabeledSpan, SourceSpan};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum LexError {
    // character
    #[error("unexpected character: '{0}'")]
    UnexpectedChar(char, SourceSpan),

    #[error("unexpected '&'")]
    UnexpectedAmpersand(SourceSpan),

    #[error("unexpected '|'")]
    UnexpectedPipe(SourceSpan),

    // string literals
    #[error("unclosed string literal")]
    UnclosedString(SourceSpan),

    #[error("invalid unicode escape")]
    InvalidUnicodeEscape(String, SourceSpan),

    #[error("invalid unicode code point")]
    InvalidUnicodeCodePoint(u32, SourceSpan),

    // number literals
    #[error("integer literal overflow")]
    IntOverflow(String, SourceSpan),

    #[error("empty integer literal")]
    EmptyIntLiteral(String, SourceSpan),

    #[error("invalid float literal")]
    InvalidFloat(String, SourceSpan),

    // instant literals
    #[error("invalid time literal")]
    InvalidTimeLiteral(SourceSpan),

    #[error("expected 'now' after @n")]
    ExpectedNow(SourceSpan),

    #[error("expected 'today' after @t")]
    ExpectedToday(SourceSpan),

    #[error("expected 'unix_' after @u")]
    ExpectedUnixPrefix(SourceSpan),

    #[error("expected unix timestamp after @unix_")]
    ExpectedUnixTimestamp(SourceSpan),

    #[error("invalid unix timestamp")]
    InvalidUnixTimestamp(String, SourceSpan),

    #[error("expected date format yyyy-mm-dd")]
    ExpectedDateFormat(SourceSpan),

    #[error("expected ':'")]
    ExpectedColon(SourceSpan),

    // date/time components
    #[error("invalid year")]
    InvalidYear(SourceSpan),

    #[error("invalid month")]
    InvalidMonth(SourceSpan),

    #[error("invalid day")]
    InvalidDay(SourceSpan),

    #[error("invalid date")]
    InvalidDate(SourceSpan),

    #[error("invalid hour")]
    InvalidHour(SourceSpan),

    #[error("invalid minute")]
    InvalidMinute(SourceSpan),
}

impl LexError {
    pub fn span(&self) -> SourceSpan {
        match self {
            LexError::UnexpectedChar(_, s)
            | LexError::UnexpectedAmpersand(s)
            | LexError::UnexpectedPipe(s)
            | LexError::UnclosedString(s)
            | LexError::IntOverflow(_, s)
            | LexError::EmptyIntLiteral(_, s)
            | LexError::InvalidFloat(_, s)
            | LexError::InvalidUnicodeEscape(_, s)
            | LexError::InvalidUnicodeCodePoint(_, s)
            | LexError::InvalidTimeLiteral(s)
            | LexError::ExpectedNow(s)
            | LexError::ExpectedToday(s)
            | LexError::ExpectedUnixPrefix(s)
            | LexError::ExpectedUnixTimestamp(s)
            | LexError::InvalidUnixTimestamp(_, s)
            | LexError::ExpectedDateFormat(s)
            | LexError::ExpectedColon(s)
            | LexError::InvalidYear(s)
            | LexError::InvalidMonth(s)
            | LexError::InvalidDay(s)
            | LexError::InvalidDate(s)
            | LexError::InvalidHour(s)
            | LexError::InvalidMinute(s) => *s,
        }
    }
}

impl Diagnostic for LexError {
    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        Some(Box::new(std::iter::once(LabeledSpan::new_with_span(
            Some("here".to_string()),
            self.span(),
        ))))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(match self {
            Self::UnexpectedChar(c, _) => Box::new(format!(
                "this character '{c}' is not valid in this position"
            )),
            Self::UnexpectedAmpersand(_) => Box::new("use && for logical AND"),
            Self::UnexpectedPipe(_) => Box::new("use |> for pipeline or || for logical OR"),
            Self::UnclosedString(_) => Box::new("add a closing \" to terminate the string literal"),
            Self::IntOverflow(lit, _) => Box::new(format!(
                "the literal `{lit}` exceeds the range of a signed 64-bit integer whose range is {}..={}",
                i64::MIN,
                i64::MAX
            )),
            Self::EmptyIntLiteral(..) => Box::new("add at least one valid digit after the prefix"),
            Self::InvalidFloat(lit, _) => {
                Box::new(format!("the literal `{lit}` is not a valid float"))
            }
            Self::InvalidUnicodeEscape(hex, _) => Box::new(format!(
                "valid escape: \\u followed by four hex digits, got \\u{hex}"
            )),
            Self::InvalidUnicodeCodePoint(cp, _) => {
                Box::new(format!("U+{cp:04X} is not a valid unicode code point"))
            }
            Self::InvalidTimeLiteral(_) => Box::new(
                "use @now, @today / @today_HH:MM:SS.SSS, @unix_SS.SSS, or @yyyy-mm-dd_HH:MM:SS.SSS",
            ),
            Self::ExpectedNow(_) => Box::new("@now gives the current instant"),
            Self::ExpectedToday(_) => Box::new("@today gives midnight of the current date"),
            Self::ExpectedUnixPrefix(_) => Box::new(
                "@unix_ must be followed by an seconds value, following the format @unix_SS.SSS",
            ),
            Self::ExpectedUnixTimestamp(_) => Box::new(
                "provide a number of seconds since the Unix epoch, may include a fractional part",
            ),
            Self::InvalidUnixTimestamp(ts, _) => Box::new(format!(
                "`{ts}` is not a valid unix timestamp (whole seconds must fit in a 64-bit signed integer)"
            )),
            Self::ExpectedDateFormat(_) => Box::new("after @, provide a date in YYYY-MM-DD format"),
            Self::ExpectedColon(_) => {
                Box::new("time components are separated by colons (HH:MM:SS)")
            }
            Self::InvalidYear(_) => Box::new("year must be a valid integer"),
            Self::InvalidMonth(_) => Box::new("month must be 1–12"),
            Self::InvalidDay(_) => Box::new("day must be 1–31 and valid for the given month/year"),
            Self::InvalidDate(_) => Box::new("this date does not exist in the Gregorian calendar"),
            Self::InvalidHour(_) => Box::new("hour must be 0–23"),
            Self::InvalidMinute(_) => Box::new("minute must be 0–59"),
        })
    }
}
