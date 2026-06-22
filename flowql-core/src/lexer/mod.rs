mod error;
mod token;

use std::borrow::Cow;
use std::ops::Range;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use miette::SourceSpan;

pub use error::LexError;
pub use token::Token;
pub use token::TokenKind;

pub struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    start: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            source,
            pos: 0,
            start: 0,
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace_and_comments();
        self.start = self.pos;

        let Some(c) = self.peek() else {
            return Ok(self.token(TokenKind::Eof));
        };

        match c {
            '(' => self.advance_and_return(TokenKind::LParen),
            ')' => self.advance_and_return(TokenKind::RParen),
            '[' => self.advance_and_return(TokenKind::LBracket),
            ']' => self.advance_and_return(TokenKind::RBracket),
            '}' => self.advance_and_return(TokenKind::RBrace),
            ',' => self.advance_and_return(TokenKind::Comma),
            ';' => self.advance_and_return(TokenKind::Semicolon),

            '+' => self.lex_plus(),
            '-' => self.advance_and_return(TokenKind::Minus),
            '*' => self.advance_and_return(TokenKind::Star),
            '/' => self.advance_and_return(TokenKind::Slash),
            '%' => self.advance_and_return(TokenKind::Percent),
            '.' => self.lex_dot(),
            ':' => self.lex_colon(),
            '=' => self.lex_eq(),
            '!' => self.lex_bang(),
            '<' => self.lex_lt(),
            '>' => self.lex_gt(),
            '&' => self.lex_amp(),
            '|' => self.lex_pipe(),
            '{' => self.advance_and_return(TokenKind::LBrace),
            '?' => self.advance_and_return(TokenKind::Question),
            '@' => self.lex_at_literal(),
            '#' => self.lex_duration(),
            '"' => self.lex_string(),
            '0'..='9' => self.lex_number(),
            c if is_ident_start(c) => self.lex_ident_or_keyword(),
            _ => {
                let span = self.single_span();
                self.pos += c.len_utf8();
                Err(LexError::UnexpectedChar(c, span))
            }
        }
    }

    // ---- helpers ----

    fn remaining(&self) -> &'a str {
        &self.source[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn peek2(&self) -> Option<char> {
        let mut chars = self.remaining().chars();
        chars.next()?;
        chars.next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.remaining().chars().next()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn advance_and_return(&mut self, kind: TokenKind) -> Result<Token, LexError> {
        self.advance();
        Ok(self.token(kind))
    }

    fn token(&self, kind: TokenKind) -> Token {
        Token::new(kind, SourceSpan::from(self.start..self.pos))
    }

    fn span_from(&self, start: usize) -> SourceSpan {
        SourceSpan::from(start..self.pos)
    }

    fn single_span(&self) -> SourceSpan {
        SourceSpan::from(self.pos..self.pos + 1)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }
                Some('/') if self.peek2() == Some('/') => {
                    self.advance();
                    self.advance();
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                Some('/') if self.peek2() == Some('*') => {
                    self.advance();
                    self.advance();
                    self.skip_block_comment();
                }
                _ => break,
            }
        }
    }

    fn skip_block_comment(&mut self) {
        while let Some(c) = self.advance() {
            match c {
                '/' if self.peek() == Some('*') => {
                    self.advance();
                    self.skip_block_comment();
                }
                '*' if self.peek() == Some('/') => {
                    self.advance();
                    return;
                }
                _ => {}
            }
        }
        // TODO: error on unclosed comment
    }

    // ---- lexers for specific token types ----

    fn lex_plus(&mut self) -> Result<Token, LexError> {
        self.advance();
        if self.peek() == Some('+') {
            self.advance();
            Ok(self.token(TokenKind::PlusPlus))
        } else {
            Ok(self.token(TokenKind::Plus))
        }
    }

    fn lex_dot(&mut self) -> Result<Token, LexError> {
        self.advance();
        if self.peek() == Some('.') {
            self.advance();
            if self.peek() == Some('=') {
                self.advance();
                Ok(self.token(TokenKind::DotDotEq))
            } else {
                Ok(self.token(TokenKind::DotDot))
            }
        } else {
            Ok(self.token(TokenKind::Dot))
        }
    }

    fn lex_colon(&mut self) -> Result<Token, LexError> {
        self.advance();
        if self.peek() == Some(':') {
            self.advance();
            Ok(self.token(TokenKind::ColonColon))
        } else {
            Ok(self.token(TokenKind::Colon))
        }
    }

    fn lex_eq(&mut self) -> Result<Token, LexError> {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            Ok(self.token(TokenKind::EqEq))
        } else {
            Ok(self.token(TokenKind::Eq))
        }
    }

    fn lex_bang(&mut self) -> Result<Token, LexError> {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            Ok(self.token(TokenKind::BangEq))
        } else {
            Ok(self.token(TokenKind::Bang))
        }
    }

    fn lex_lt(&mut self) -> Result<Token, LexError> {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            Ok(self.token(TokenKind::LAngleEq))
        } else {
            Ok(self.token(TokenKind::LAngle))
        }
    }

    fn lex_gt(&mut self) -> Result<Token, LexError> {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            Ok(self.token(TokenKind::RAngleEq))
        } else {
            Ok(self.token(TokenKind::RAngle))
        }
    }

    fn lex_amp(&mut self) -> Result<Token, LexError> {
        self.advance();
        if self.peek() == Some('&') {
            self.advance();
            Ok(self.token(TokenKind::AmpAmp))
        } else {
            Err(LexError::UnexpectedAmpersand(SourceSpan::from(
                self.pos - 1..self.pos,
            )))
        }
    }

    fn lex_pipe(&mut self) -> Result<Token, LexError> {
        self.advance();
        if self.peek() == Some('>') {
            self.advance();
            Ok(self.token(TokenKind::PipeR))
        } else if self.peek() == Some('|') {
            self.advance();
            Ok(self.token(TokenKind::PipePipe))
        } else {
            Err(LexError::UnexpectedPipe(SourceSpan::from(
                self.pos - 1..self.pos,
            )))
        }
    }

    fn lex_ident_or_keyword(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if is_ident_continue(c) {
                self.advance();
            } else {
                break;
            }
        }
        let ident = &self.source[start..self.pos];
        let kind = match ident {
            "let" => TokenKind::Let,
            "table" => TokenKind::Table,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Ident(ident.to_string()),
        };
        Ok(self.token(kind))
    }

    fn lex_number(&mut self) -> Result<Token, LexError> {
        let start = self.pos;

        if self.peek() == Some('0') {
            match self.peek2() {
                Some('x') | Some('X') => return self.lex_hex_int(),
                Some('o') | Some('O') => return self.lex_octal_int(),
                Some('b') | Some('B') => return self.lex_binary_int(),
                _ => {}
            }
        }

        self.read_digits();

        let has_dot = self.peek() == Some('.') && self.peek2().is_some_and(|c| c.is_ascii_digit());

        if has_dot {
            self.advance();
            self.read_digits();
            if let Some('e') | Some('E') = self.peek() {
                self.lex_float_exponent();
            }
            let raw = &self.source[start..self.pos];
            let clean = Self::clean_int_prefix(raw, "");
            let value: f64 = clean.parse().map_err(|_| {
                LexError::InvalidFloat(raw.to_string(), SourceSpan::from(start..self.pos))
            })?;
            Ok(self.token(TokenKind::Float(value)))
        } else if let Some('e') | Some('E') = self.peek() {
            self.lex_float_exponent();
            let raw = &self.source[start..self.pos];
            let clean = Self::clean_int_prefix(raw, "");
            let value: f64 = clean.parse().map_err(|_| {
                LexError::InvalidFloat(raw.to_string(), SourceSpan::from(start..self.pos))
            })?;
            Ok(self.token(TokenKind::Float(value)))
        } else {
            let raw = &self.source[start..self.pos];
            let clean = Self::clean_int_prefix(raw, "");
            let value: i64 = clean.parse().map_err(|_| {
                LexError::IntOverflow(raw.to_string(), SourceSpan::from(start..self.pos))
            })?;
            Ok(self.token(TokenKind::Int(value)))
        }
    }

    fn lex_hex_int(&mut self) -> Result<Token, LexError> {
        self.lex_int_radix(16, "0x", |c| c.is_ascii_hexdigit() || c == '_')
    }

    fn lex_octal_int(&mut self) -> Result<Token, LexError> {
        self.lex_int_radix(8, "0o", |c| {
            c.is_ascii_digit() && c != '8' && c != '9' || c == '_'
        })
    }

    fn lex_binary_int(&mut self) -> Result<Token, LexError> {
        self.lex_int_radix(2, "0b", |c| c == '0' || c == '1' || c == '_')
    }

    fn lex_int_radix<F>(&mut self, radix: u32, prefix: &str, is_digit: F) -> Result<Token, LexError>
    where
        F: Fn(char) -> bool,
    {
        let start = self.pos;
        for _ in 0..prefix.chars().count() {
            self.advance();
        }
        while let Some(c) = self.peek()
            && is_digit(c)
        {
            self.advance();
        }
        let raw = &self.source[start..self.pos];
        let clean = Self::clean_int_prefix(raw, prefix);
        if clean.is_empty() {
            return Err(LexError::EmptyIntLiteral(
                raw.to_string(),
                SourceSpan::from(start..self.pos),
            ));
        }
        let value = i64::from_str_radix(&clean, radix).map_err(|_| {
            LexError::IntOverflow(raw.to_string(), SourceSpan::from(start..self.pos))
        })?;
        Ok(self.token(TokenKind::Int(value)))
    }

    fn clean_int_prefix(raw: &'a str, prefix: &str) -> Cow<'a, str> {
        let clean = raw.strip_prefix(prefix).unwrap_or(raw);
        if clean.contains('_') {
            Cow::Owned(clean.replace('_', ""))
        } else {
            Cow::Borrowed(clean)
        }
    }

    fn lex_float_exponent(&mut self) {
        self.advance();
        if let Some('+') | Some('-') = self.peek() {
            self.advance();
        }
        self.read_digits();
    }

    fn read_digits(&mut self) {
        while let Some(c) = self.peek()
            && (c.is_ascii_digit() || c == '_')
        {
            self.advance();
        }
    }

    fn lex_string(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        self.advance();
        let mut value = String::new();

        loop {
            match self.advance() {
                None | Some('\n') => {
                    return Err(LexError::UnclosedString(SourceSpan::from(start..self.pos)));
                }
                Some('"') => {
                    break;
                }
                Some('\\') => match self.advance() {
                    None => {
                        return Err(LexError::UnclosedString(SourceSpan::from(start..self.pos)));
                    }
                    // allow string continuation on new line
                    Some('\n') => {}
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some('r') => value.push('\r'),
                    Some('0') => value.push('\0'),
                    Some('u') => {
                        let escape_start = self.pos - 2;
                        if self.peek() != Some('{') {
                            return Err(LexError::InvalidUnicodeEscape(SourceSpan::from(
                                escape_start..self.pos,
                            )));
                        }
                        self.advance();
                        let hex_start = self.pos;
                        while self.peek().is_some_and(|c| c.is_ascii_hexdigit()) {
                            self.advance();
                        }
                        let hex = &self.source[hex_start..self.pos];
                        if hex.is_empty() || hex.len() > 6 {
                            return Err(LexError::InvalidUnicodeEscape(SourceSpan::from(
                                escape_start..self.pos,
                            )));
                        }
                        if self.peek() != Some('}') {
                            return Err(LexError::InvalidUnicodeEscape(SourceSpan::from(
                                escape_start..self.pos,
                            )));
                        }
                        self.advance();
                        let code = u32::from_str_radix(hex, 16).map_err(|_| {
                            LexError::InvalidUnicodeEscape(SourceSpan::from(escape_start..self.pos))
                        })?;
                        let c = char::from_u32(code).ok_or_else(|| {
                            LexError::InvalidUnicodeCodePoint(
                                code,
                                SourceSpan::from(escape_start..self.pos),
                            )
                        })?;
                        value.push(c);
                    }
                    Some(ch) => {
                        return Err(LexError::InvalidEscape(
                            ch,
                            SourceSpan::from(self.pos - ch.len_utf8() - 1..self.pos),
                        ));
                    }
                },
                Some(c) => value.push(c),
            }
        }

        Ok(self.token(TokenKind::Str(value)))
    }

    fn lex_at_literal(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        self.advance(); // consume '@'

        match self.peek() {
            Some('n') => self.lex_at_now(start),
            Some('t') => self.lex_at_today(start),
            Some('u') => self.lex_at_unix(start),
            Some(c) if c.is_ascii_digit() || c == '+' || c == '-' => self.lex_at_date(start),
            _ => Err(LexError::InvalidTimeLiteral(self.span_from(start))),
        }
    }

    fn lex_at_now(&mut self, start: usize) -> Result<Token, LexError> {
        for ch in "now".chars() {
            match self.advance() {
                Some(c) if c == ch => {}
                _ => {
                    return Err(LexError::ExpectedNow(self.span_from(start)));
                }
            }
        }
        if self.peek().is_some_and(is_ident_continue) {
            return Err(LexError::ExpectedNow(self.span_from(start)));
        }
        Ok(self.token(TokenKind::Now))
    }

    fn lex_at_today(&mut self, start: usize) -> Result<Token, LexError> {
        for ch in "today".chars() {
            match self.advance() {
                Some(c) if c == ch => {}
                _ => {
                    return Err(LexError::ExpectedToday(self.span_from(start)));
                }
            }
        }

        match self.read_time_suffix(start)? {
            Some((hour, minute, secs, nanos)) => {
                Ok(self.token(TokenKind::TodayAt(hour, minute, secs, nanos)))
            }
            None => {
                if self.peek().is_some_and(is_ident_continue) {
                    return Err(LexError::ExpectedToday(self.span_from(start)));
                }
                Ok(self.token(TokenKind::Today))
            }
        }
    }

    fn read_time_suffix(&mut self, start: usize) -> Result<Option<(u32, u32, u32, u32)>, LexError> {
        if self.peek() != Some('_') {
            return Ok(None);
        }
        self.advance();
        let hour = self.source[self.advance_digits()]
            .parse::<u32>()
            .map_err(|_| LexError::InvalidHour(self.span_from(start)))?;
        if hour > 23 {
            return Err(LexError::InvalidHour(self.span_from(start)));
        }
        match self.peek() {
            Some(':') => {
                self.advance();
            }
            _ => {
                return Err(LexError::ExpectedColon(self.span_from(start)));
            }
        }
        let minute = self.source[self.advance_digits()]
            .parse::<u32>()
            .map_err(|_| LexError::InvalidMinute(self.span_from(start)))?;
        if minute > 59 {
            return Err(LexError::InvalidMinute(self.span_from(start)));
        }
        let mut secs = 0u32;
        let mut nanos = 0u32;
        if self.peek() == Some(':') {
            self.advance();
            let sec_range = self.advance_digits();
            if self.peek() == Some('.') {
                self.advance();
                let frac_range = self.advance_digits();
                (secs, nanos) = split_seconds(&self.source[sec_range.start..frac_range.end]);
            } else {
                secs = self.source[sec_range].parse().unwrap_or(0);
            }
        }
        Ok(Some((hour, minute, secs, nanos)))
    }

    fn lex_at_unix(&mut self, start: usize) -> Result<Token, LexError> {
        for ch in "unix_".chars() {
            match self.advance() {
                Some(c) if c == ch => {}
                _ => {
                    return Err(LexError::ExpectedUnixPrefix(self.span_from(start)));
                }
            }
        }
        let negative = self.peek() == Some('-');
        if let Some('+') | Some('-') = self.peek() {
            self.advance();
        }
        let start = self.pos;
        while let Some(c) = self.peek()
            && c.is_ascii_digit()
        {
            self.advance();
        }
        let sec_str = &self.source[start..self.pos];

        if sec_str.is_empty() {
            return Err(LexError::ExpectedUnixTimestamp(self.span_from(start)));
        }
        let secs: i64 = sec_str.parse().map_err(|_| {
            LexError::InvalidUnixTimestamp(sec_str.to_string(), self.span_from(start))
        })?;
        let secs = if negative { -secs } else { secs };

        let nanos = if self.peek() == Some('.') {
            self.advance();
            let frac = self.advance_digits();
            parse_nanos(&self.source[frac])
        } else {
            0
        };

        let (secs, nanos) = if negative && nanos > 0 {
            (secs - 1, 1_000_000_000 - nanos)
        } else {
            (secs, nanos)
        };

        Ok(self.token(TokenKind::Instant(secs, nanos)))
    }

    fn lex_at_date(&mut self, start: usize) -> Result<Token, LexError> {
        let negative = self.peek() == Some('-');
        if let Some('+') | Some('-') = self.peek() {
            self.advance();
        }
        let y_range = self.advance_digits();

        match self.peek() {
            Some('-') => {
                self.advance();
            }
            _ => {
                return Err(LexError::ExpectedDateFormat(self.span_from(start)));
            }
        }
        let m_range = self.advance_digits();
        match self.peek() {
            Some('-') => {
                self.advance();
            }
            _ => {
                return Err(LexError::ExpectedDateFormat(self.span_from(start)));
            }
        }
        let d_range = self.advance_digits();

        let (hour, minute, secs, nanos) = self.read_time_suffix(start)?.unwrap_or((0, 0, 0, 0));

        let year: i32 = self.source[y_range]
            .parse()
            .map_err(|_| LexError::InvalidYear(self.span_from(start)))?;
        let year = if negative { -year } else { year };
        let month: u32 = self.source[m_range]
            .parse()
            .map_err(|_| LexError::InvalidMonth(self.span_from(start)))?;
        if !(1..=12).contains(&month) {
            return Err(LexError::InvalidMonth(self.span_from(start)));
        }
        let day: u32 = self.source[d_range]
            .parse()
            .map_err(|_| LexError::InvalidDay(self.span_from(start)))?;
        if !(1..=31).contains(&day) {
            return Err(LexError::InvalidDay(self.span_from(start)));
        }

        let date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| LexError::InvalidDate(self.span_from(start)))?;
        let time = NaiveTime::from_hms_nano_opt(hour, minute, secs, nanos)
            .ok_or_else(|| LexError::InvalidDate(self.span_from(start)))?;
        let dt = NaiveDateTime::new(date, time);
        let total_secs = dt.and_utc().timestamp();
        Ok(self.token(TokenKind::Instant(total_secs, nanos)))
    }

    fn lex_duration(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        self.advance(); // consume '#'

        let negative = self.peek() == Some('-');
        if let Some('+') | Some('-') = self.peek() {
            self.advance();
        }

        let mut weeks: bool = false;
        let mut days: bool = false;
        let mut hours: bool = false;
        let mut minutes: bool = false;
        let mut secs: bool = false;

        let mut nanos: u32 = 0;
        let mut total_secs: i64 = 0;

        loop {
            let int_range = self.advance_digits();
            if int_range.is_empty() {
                if self.peek() == Some('.') {
                    return Err(LexError::ExpectedDurationDigits(self.span_from(start)));
                } else {
                    break;
                }
            }
            let int_part: i64 = self.source[int_range].parse().unwrap();

            let frac_part = if self.peek() == Some('.') {
                self.advance();
                let frac_range = self.advance_digits();
                if frac_range.is_empty() {
                    return Err(LexError::IncompleteDurationFraction(self.span_from(start)));
                }
                Some(parse_nanos(&self.source[frac_range]))
            } else {
                None
            };

            let suffix = match self.peek() {
                Some(c @ ('w' | 'd' | 'h' | 'm' | 's')) => {
                    self.advance();
                    c
                }
                Some(c) => {
                    return Err(LexError::UnexpectedDurationSuffix(c, self.span_from(start)));
                }
                None => {
                    return Err(LexError::MissingDurationSuffix(self.span_from(start)));
                }
            };

            if frac_part.is_some() && suffix != 's' {
                return Err(LexError::FractionalNotAllowed(
                    suffix,
                    self.span_from(start),
                ));
            }

            let (slot, multiplier) = match suffix {
                'w' => (&mut weeks, 7 * 24 * 60 * 60),
                'd' => (&mut days, 24 * 60 * 60),
                'h' => (&mut hours, 60 * 60),
                'm' => (&mut minutes, 60),
                's' => {
                    nanos = frac_part.unwrap_or(0);
                    (&mut secs, 1)
                }
                _ => unreachable!(),
            };
            if *slot {
                return Err(LexError::DuplicateDurationComponent(
                    suffix,
                    self.span_from(start),
                ));
            }
            *slot = true;
            total_secs = total_secs
                .checked_add(
                    int_part
                        .checked_mul(multiplier)
                        .ok_or(LexError::DurationOverflow(self.span_from(start)))?,
                )
                .ok_or(LexError::DurationOverflow(self.span_from(start)))?;
        }

        if !(weeks || days || hours || minutes || secs) {
            return Err(LexError::EmptyDuration(self.span_from(start)));
        }

        if negative {
            total_secs = -total_secs
        }

        Ok(self.token(TokenKind::Duration(total_secs, nanos)))
    }

    fn advance_digits(&mut self) -> Range<usize> {
        let start = self.pos;
        while let Some(c) = self.peek()
            && c.is_ascii_digit()
        {
            self.advance();
        }
        start..self.pos
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn parse_nanos(frac_str: &str) -> u32 {
    let mut padded = String::from(frac_str);
    while padded.len() < 9 {
        padded.push('0');
    }
    if padded.len() > 9 {
        padded = padded[..9].to_string();
    }
    padded.parse().unwrap_or(0)
}

fn split_seconds(s: &str) -> (u32, u32) {
    if let Some(dot_pos) = s.find('.') {
        let whole: u32 = s[..dot_pos].parse().unwrap_or(0);
        let nanos = parse_nanos(&s[dot_pos + 1..]);
        (whole, nanos)
    } else {
        let whole: u32 = s.parse().unwrap_or(0);
        (whole, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_all(source: &str) -> Vec<TokenKind> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token().unwrap();
            let kind = token.kind.clone();
            if kind == TokenKind::Eof {
                break;
            }
            tokens.push(kind);
        }
        tokens
    }

    fn check_tokens(source: &str, expected: &[TokenKind]) {
        let actual = lex_all(source);
        assert_eq!(actual, expected, "source: {:?}", source);
    }

    #[test]
    fn test_keywords() {
        check_tokens("let", &[TokenKind::Let]);
        check_tokens("table", &[TokenKind::Table]);
        check_tokens("if", &[TokenKind::If]);
        check_tokens("else", &[TokenKind::Else]);
        check_tokens("true", &[TokenKind::True]);
        check_tokens("false", &[TokenKind::False]);
    }

    #[test]
    fn test_identifier() {
        check_tokens("hello", &[TokenKind::Ident("hello".into())]);
        check_tokens("user_name", &[TokenKind::Ident("user_name".into())]);
        check_tokens("_internal", &[TokenKind::Ident("_internal".into())]);
        check_tokens("join", &[TokenKind::Ident("join".into())]);
        check_tokens("Some", &[TokenKind::Ident("Some".into())]);
        check_tokens("None", &[TokenKind::Ident("None".into())]);
    }

    #[test]
    fn test_keyword_not_in_identifier() {
        check_tokens("letter", &[TokenKind::Ident("letter".into())]);
    }

    #[test]
    fn test_integers() {
        check_tokens("42", &[TokenKind::Int(42)]);
        check_tokens("0", &[TokenKind::Int(0)]);
        check_tokens("-5", &[TokenKind::Minus, TokenKind::Int(5)]);
        check_tokens("1_000", &[TokenKind::Int(1000)]);
        check_tokens("0xDEAD", &[TokenKind::Int(0xDEAD)]);
        check_tokens("0o755", &[TokenKind::Int(0o755)]);
        check_tokens("0b1101", &[TokenKind::Int(0b1101)]);
        check_tokens("0b1101_0010", &[TokenKind::Int(0b1101_0010)]);
    }

    #[test]
    fn test_floats() {
        check_tokens("3.21", &[TokenKind::Float(3.21)]);
        check_tokens("0.001", &[TokenKind::Float(0.001)]);
        check_tokens("1e10", &[TokenKind::Float(1e10)]);
        check_tokens("1.5e-3", &[TokenKind::Float(1.5e-3)]);
        check_tokens("1.5e+3", &[TokenKind::Float(1500.0)]);
    }

    #[test]
    fn test_strings() {
        check_tokens("\"hello\"", &[TokenKind::Str("hello".into())]);
        check_tokens("\"hello world\"", &[TokenKind::Str("hello world".into())]);
        check_tokens(
            r#""escaped \"quote\"""#,
            &[TokenKind::Str(r#"escaped "quote""#.into())],
        );
    }

    #[test]
    fn test_string_continuation() {
        check_tokens("\"hello\\\nworld\"", &[TokenKind::Str("helloworld".into())]);
    }

    #[test]
    fn test_invalid_escape() {
        let mut lexer = Lexer::new(r#""\z""#);
        assert!(lexer.next_token().is_err());
    }

    #[test]
    fn test_unicode_escape_basic() {
        check_tokens(r#""\u{41}""#, &[TokenKind::Str("A".into())]);
        check_tokens(r#""\u{3A}""#, &[TokenKind::Str(":".into())]);
        check_tokens(r#""\u{1F600}""#, &[TokenKind::Str("\u{1F600}".into())]);
        check_tokens(r#""\u{10FFFF}""#, &[TokenKind::Str("\u{10FFFF}".into())]);
    }

    #[test]
    fn test_unicode_escape_errors() {
        let cases = &[
            r#""\u{}""#,
            r#""\u{1234567}""#,
            r#""\u{110000}""#,
            r#""\uX""#,
        ];
        for &src in cases {
            let mut lexer = Lexer::new(src);
            assert!(lexer.next_token().is_err(), "expected error for: {src}");
        }
    }

    #[test]
    fn test_operators() {
        check_tokens("+", &[TokenKind::Plus]);
        check_tokens("-", &[TokenKind::Minus]);
        check_tokens("*", &[TokenKind::Star]);
        check_tokens("/", &[TokenKind::Slash]);
        check_tokens("%", &[TokenKind::Percent]);
        check_tokens("==", &[TokenKind::EqEq]);
        check_tokens("!=", &[TokenKind::BangEq]);
        check_tokens("<", &[TokenKind::LAngle]);
        check_tokens("<=", &[TokenKind::LAngleEq]);
        check_tokens(">", &[TokenKind::RAngle]);
        check_tokens(">=", &[TokenKind::RAngleEq]);
        check_tokens("&&", &[TokenKind::AmpAmp]);
        check_tokens("||", &[TokenKind::PipePipe]);
        check_tokens("!", &[TokenKind::Bang]);
        check_tokens("++", &[TokenKind::PlusPlus]);
        check_tokens("..", &[TokenKind::DotDot]);
        check_tokens("..=", &[TokenKind::DotDotEq]);
        check_tokens("|>", &[TokenKind::PipeR]);
        check_tokens("::", &[TokenKind::ColonColon]);
        check_tokens("=", &[TokenKind::Eq]);
        check_tokens(".", &[TokenKind::Dot]);
        check_tokens(":", &[TokenKind::Colon]);
    }

    #[test]
    fn test_delimiters() {
        check_tokens("(", &[TokenKind::LParen]);
        check_tokens(")", &[TokenKind::RParen]);
        check_tokens("[", &[TokenKind::LBracket]);
        check_tokens("]", &[TokenKind::RBracket]);
        check_tokens("{", &[TokenKind::LBrace]);
        check_tokens("}", &[TokenKind::RBrace]);
        check_tokens(";", &[TokenKind::Semicolon]);
        check_tokens(",", &[TokenKind::Comma]);
        check_tokens("?", &[TokenKind::Question]);
    }

    #[test]
    fn test_whitespace() {
        check_tokens("let   x", &[TokenKind::Let, TokenKind::Ident("x".into())]);
        check_tokens("let\nx", &[TokenKind::Let, TokenKind::Ident("x".into())]);
        check_tokens("let\tx", &[TokenKind::Let, TokenKind::Ident("x".into())]);
    }

    #[test]
    fn test_line_comment() {
        check_tokens(
            "let // this is a comment\nx",
            &[TokenKind::Let, TokenKind::Ident("x".into())],
        );
    }

    #[test]
    fn test_block_comment() {
        check_tokens(
            "let /* comment */ x",
            &[TokenKind::Let, TokenKind::Ident("x".into())],
        );
    }

    #[test]
    fn test_nested_block_comment() {
        check_tokens(
            "let /* outer /* inner */ still */ x",
            &[TokenKind::Let, TokenKind::Ident("x".into())],
        );
    }

    #[test]
    fn test_pipeline_expression() {
        check_tokens(
            "users |> [ name, age ]",
            &[
                TokenKind::Ident("users".into()),
                TokenKind::PipeR,
                TokenKind::LBracket,
                TokenKind::Ident("name".into()),
                TokenKind::Comma,
                TokenKind::Ident("age".into()),
                TokenKind::RBracket,
            ],
        );
    }

    #[test]
    fn test_filter_syntax() {
        check_tokens(
            "users |> [? age > 18 ]",
            &[
                TokenKind::Ident("users".into()),
                TokenKind::PipeR,
                TokenKind::LBracket,
                TokenKind::Question,
                TokenKind::Ident("age".into()),
                TokenKind::RAngle,
                TokenKind::Int(18),
                TokenKind::RBracket,
            ],
        );
    }

    #[test]
    fn test_join_syntax() {
        check_tokens(
            "users |> join<orders>(id == orders::user_id) {: total = orders::amount }",
            &[
                TokenKind::Ident("users".into()),
                TokenKind::PipeR,
                TokenKind::Ident("join".into()),
                TokenKind::LAngle,
                TokenKind::Ident("orders".into()),
                TokenKind::RAngle,
                TokenKind::LParen,
                TokenKind::Ident("id".into()),
                TokenKind::EqEq,
                TokenKind::Ident("orders".into()),
                TokenKind::ColonColon,
                TokenKind::Ident("user_id".into()),
                TokenKind::RParen,
                TokenKind::LBrace,
                TokenKind::Colon,
                TokenKind::Ident("total".into()),
                TokenKind::Eq,
                TokenKind::Ident("orders".into()),
                TokenKind::ColonColon,
                TokenKind::Ident("amount".into()),
                TokenKind::RBrace,
            ],
        );
    }

    #[test]
    fn test_typographic_operators() {
        let result = Lexer::new("&").next_token();
        assert!(result.is_err());
    }

    #[test]
    fn test_at_now() {
        let tokens = lex_all("@now");
        assert_eq!(tokens, &[TokenKind::Now]);
    }

    #[test]
    fn test_at_now_error_on_extra_chars() {
        let mut lexer = Lexer::new("@nows");
        assert!(lexer.next_token().is_err());
    }

    #[test]
    fn test_at_today() {
        let tokens = lex_all("@today");
        assert_eq!(tokens, &[TokenKind::Today]);
    }

    #[test]
    fn test_at_today_with_time() {
        let tokens = lex_all("@today_14:30:00");
        assert_eq!(tokens, &[TokenKind::TodayAt(14, 30, 0, 0)]);
    }

    #[test]
    fn test_at_today_with_fractional_seconds() {
        let tokens = lex_all("@today_14:30:00.123");
        assert_eq!(tokens, &[TokenKind::TodayAt(14, 30, 0, 123000000)]);
    }

    #[test]
    fn test_at_today_error_on_extra_chars() {
        let mut lexer = Lexer::new("@todayy");
        assert!(lexer.next_token().is_err());
    }

    #[test]
    fn test_at_today_error_on_bare_underscore() {
        let mut lexer = Lexer::new("@today_");
        assert!(lexer.next_token().is_err());
    }

    #[test]
    fn test_at_unix() {
        let tokens = lex_all("@unix_1717115400");
        assert_eq!(tokens, &[TokenKind::Instant(1717115400, 0)]);
    }

    #[test]
    fn test_at_unix_with_fractional() {
        let tokens = lex_all("@unix_1717115400.123");
        assert_eq!(tokens, &[TokenKind::Instant(1717115400, 123000000)]);
    }

    #[test]
    fn test_at_unix_error_empty() {
        let mut lexer = Lexer::new("@unix_");
        assert!(lexer.next_token().is_err());
    }

    #[test]
    fn test_at_unix_error_bad_prefix() {
        let mut lexer = Lexer::new("@unixx_123");
        assert!(lexer.next_token().is_err());
    }

    #[test]
    fn test_at_unix_negative_no_fraction() {
        let tokens = lex_all("@unix_-5");
        assert_eq!(tokens, &[TokenKind::Instant(-5, 0)]);
    }

    #[test]
    fn test_at_unix_positive_sign() {
        let tokens = lex_all("@unix_+7.25");
        assert_eq!(tokens, &[TokenKind::Instant(7, 250000000)]);
    }

    #[test]
    fn test_at_unix_negative_with_fraction() {
        let tokens = lex_all("@unix_-1.2");
        assert_eq!(tokens, &[TokenKind::Instant(-2, 800000000)]);
    }

    #[test]
    fn test_at_unix_negative_zero_whole() {
        let tokens = lex_all("@unix_-0.5");
        assert_eq!(tokens, &[TokenKind::Instant(-1, 500000000)]);
    }

    #[test]
    fn test_instant_literal() {
        let tokens = lex_all("@2026-05-31");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], TokenKind::Instant(_, _)));
    }

    #[test]
    fn test_instant_with_time() {
        let tokens = lex_all("@2026-05-31_14:30:00");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], TokenKind::Instant(_, _)));
    }

    #[test]
    fn test_instant_negative_year() {
        let tokens = lex_all("@-2024-05-31");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], TokenKind::Instant(_, _)));
    }

    #[test]
    fn test_instant_positive_sign_year() {
        let tokens = lex_all("@+2024-05-31");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], TokenKind::Instant(_, _)));
    }

    #[test]
    fn test_instant_negative_year_with_time() {
        let tokens = lex_all("@-2000-01-15_06:30:00");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], TokenKind::Instant(_, _)));
    }

    #[test]
    fn test_instants_epoch_sign() {
        if let TokenKind::Instant(secs, _) = lex_all("@2026-05-31")[0] {
            assert!(secs > 0, "year 2026 should be after 1970");
        }
        if let TokenKind::Instant(secs, _) = lex_all("@-1000-01-01")[0] {
            assert!(secs < 0, "year -1000 should be before 1970");
        }
    }

    #[test]
    fn test_span_tracking() {
        let mut lexer = Lexer::new("let x");
        let t1 = lexer.next_token().unwrap();
        assert_eq!(t1.span, SourceSpan::from(0..3));
        let t2 = lexer.next_token().unwrap();
        assert_eq!(t2.span, SourceSpan::from(4..5));
    }

    #[test]
    fn test_at_not_identifier() {
        let mut lexer = Lexer::new("@not_a_time");
        let result = lexer.next_token();
        assert!(result.is_err());
    }

    #[test]
    fn test_duration_all_components() {
        let tokens = lex_all("#3w4d5h6m7s");
        assert_eq!(tokens, &[TokenKind::Duration(2178367, 0)]);
    }

    #[test]
    fn test_duration_fractional_seconds() {
        let tokens = lex_all("#7.89s");
        assert_eq!(tokens, &[TokenKind::Duration(7, 890000000)]);
    }

    #[test]
    fn test_duration_single_large() {
        let tokens = lex_all("#500000d");
        assert_eq!(tokens, &[TokenKind::Duration(43200000000, 0)]);
    }

    #[test]
    fn test_duration_subset() {
        let tokens = lex_all("#3d4h5m");
        assert_eq!(tokens, &[TokenKind::Duration(273900, 0)]);
    }

    #[test]
    fn test_duration_errors() {
        let cases = &[
            "#", "#1x", "#1d2d", "#1.5d", "#1.5s2s", "#5.s", "#.5s", "#1y", "#1M", "#1H", "#1S",
        ];
        for &src in cases {
            let mut lexer = Lexer::new(src);
            assert!(lexer.next_token().is_err(), "expected error for: {src}");
        }
    }
}
