use std::{ops::Range, str::CharIndices};

pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
}

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

pub enum TokenKind<'a> {
    // end of file
    Eof,

    // identifier
    Ident(&'a str),

    // literals
    IntLit(i64),
    FloatLit(f64),
    StringLit(&'a str),

    // punctuation
    Semicolon,
    Colon,
    DoubleColon,
    Comma,
    Dot,
    TripleDot,

    // delimiters
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    LogicalOr,
    LogicalAnd,
    Pipeline,

    // comparisons
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Assign,

    // keywords
    If,
    Let,
    Store,
    Drop,
}

pub enum LexerError {
    UnknownCharacter,
    IncompleteToken,
    UnclosedComment,
    MalformedFloat,
    InternalError,
}

impl<'a> TokenKind<'a> {
    pub fn is_keyword(&self) -> bool {
        matches!(self, Self::If | Self::Let | Self::Store | Self::Drop)
    }

    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Self::IntLit(_) | Self::FloatLit(_) | Self::StringLit(_)
        )
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Result<Token<'a>, LexerError> {
        let Some((_, ch)) = self.chars.peek() else {
            return Ok(Token {
                kind: TokenKind::Eof,
                span: Span::new(0, 0),
            });
        };

        let token = match ch {
            '_' => self.lex_ident(),
            _ if ch.is_ascii_alphabetic() => self.lex_ident(),
            _ if ch.is_ascii_digit() => self.lex_numeric()?,
            '+' => self.lex_single(TokenKind::Plus),
            '-' => self.lex_single(TokenKind::Minus),
            '*' => self.lex_single(TokenKind::Star),
            '/' => self.lex_single(TokenKind::Slash),
            '%' => self.lex_single(TokenKind::Percent),
            '!' => self.lex_alternative(TokenKind::Bang, '=', TokenKind::NotEqual),
            '=' => self.lex_alternative(TokenKind::Assign, '=', TokenKind::Equal),
            '>' => self.lex_alternative(TokenKind::Greater, '=', TokenKind::GreaterEqual),
            '<' => self.lex_alternative(TokenKind::Less, '=', TokenKind::LessEqual),
            ';' => self.lex_single(TokenKind::Semicolon),
            ':' => self.lex_alternative(TokenKind::Colon, ':', TokenKind::DoubleColon),
            ',' => self.lex_single(TokenKind::Comma),
            '.' => self.lex_dot(),
            '(' => self.lex_single(TokenKind::LParen),
            ')' => self.lex_single(TokenKind::RParen),
            '[' => self.lex_single(TokenKind::LBracket),
            ']' => self.lex_single(TokenKind::RBracket),
            '{' => self.lex_single(TokenKind::LBrace),
            '}' => self.lex_single(TokenKind::RBrace),
            other => todo!(),
        };

        self.consume_whitespace();

        todo!()
    }

    fn consume_whitespace(&mut self) {
        // TODO: add support for comments

        while let Some((_, ch)) = self.chars.peek()
            && ch.is_ascii_whitespace()
        {
            self.chars.next();
        }
    }

    fn consume_while(&mut self, mut condition: impl FnMut(&char) -> bool) -> Option<Span> {
        let mut end = 0;
        let mut start = None;

        while let Some((i, ch)) = self.chars.peek() {
            if condition(ch) {
                if start.is_none() {
                    start = Some(*i);
                }
                end = i + ch.len_utf8();
                self.chars.next();
            } else {
                break;
            }
        }

        start.map(|s| Span::new(s, end))
    }

    fn lex_single(&mut self, kind: TokenKind<'a>) -> Token<'a> {
        let (start, ch) = self.chars.next().unwrap();

        Token {
            kind,
            span: Span::new(start, start + ch.len_utf8()),
        }
    }

    fn lex_alternative(
        &mut self,
        current: TokenKind<'a>,
        expected: char,
        alternative: TokenKind<'a>,
    ) -> Token<'a> {
        let (start, ch) = self.chars.next().unwrap();
        if let Some((i, ch)) = self.chars.peek()
            && *ch == expected
        {
            return Token {
                kind: alternative,
                span: Span::new(start, i + ch.len_utf8()),
            };
        }

        Token {
            kind: current,
            span: Span::new(start, start + ch.len_utf8()),
        }
    }

    fn lex_ident(&mut self) -> Token<'a> {
        let span = self
            .consume_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
            .unwrap();
        let ident = &self.input[span.range()];

        // TODO: match keywords

        Token {
            kind: TokenKind::Ident(ident),
            span,
        }
    }

    fn lex_numeric(&mut self) -> Result<Token<'a>, LexerError> {
        let int = self.consume_while(char::is_ascii_digit).unwrap();

        let decimal = if let Some((_, ch)) = self.chars.peek()
            && *ch == '.'
        {
            self.chars.next();
            let span = self
                .consume_while(char::is_ascii_digit)
                .ok_or(LexerError::MalformedFloat)?;
            Some(span)
        } else {
            None
        };

        let exponent = if let Some((_, ch)) = self.chars.peek()
            && (*ch == 'e' || *ch == 'E')
        {
            self.chars.next();

            // handle negative exponent
            if let Some((_, ch)) = self.chars.peek()
                && *ch == '-'
            {
                self.chars.next();
            }

            let span = self
                .consume_while(char::is_ascii_digit)
                .ok_or(LexerError::MalformedFloat)?;
            Some(span)
        } else {
            None
        };

        // number is an integer
        if decimal.is_none() && exponent.is_none() {
            return Ok(Token {
                kind: TokenKind::IntLit(
                    self.input[int.range()]
                        .parse()
                        .map_err(|_| LexerError::InternalError)?,
                ),
                span: int,
            });
        }

        let mut span = int;

        if let Some(dec) = decimal {
            span = span.merge(&dec);
        }
        if let Some(exp) = exponent {
            span = span.merge(&exp);
        }

        Ok(Token {
            kind: TokenKind::FloatLit(
                self.input[span.range()]
                    .parse()
                    .map_err(|_| LexerError::InternalError)?,
            ),
            span,
        })
    }

    fn lex_dot(&mut self) -> Token<'a> {
        let (start, ch) = self.chars.next().unwrap();
        if let Some((_, ch)) = self.chars.peek()
            && *ch == '.'
        {
            self.chars.next();
            if let Some((_, ch)) = self.chars.peek()
                && *ch == '.'
            {
                let (i, ch) = self.chars.next().unwrap();
                return Token {
                    kind: TokenKind::TripleDot,
                    span: Span::new(start, i + ch.len_utf8()),
                };
            }
        }

        Token {
            kind: TokenKind::Dot,
            span: Span::new(start, start + ch.len_utf8()),
        }
    }
}
