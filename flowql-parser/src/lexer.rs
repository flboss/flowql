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

// TODO: add info to error variants
pub enum LexerError {
    UnknownCharacter,
    IncompleteToken,
    UnclosedComment,
    UnclosedStringLit,
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

struct Lookahead<I: Iterator> {
    iter: I,
    buffer: [Option<I::Item>; 2],
}

impl<I: Iterator> Lookahead<I> {
    fn new(mut iter: I) -> Self {
        let first = iter.next();
        let second = iter.next();

        Self {
            iter,
            buffer: [first, second],
        }
    }

    fn peek(&self) -> Option<&I::Item> {
        self.buffer[0].as_ref()
    }

    fn peek_2(&self) -> Option<&I::Item> {
        self.buffer[1].as_ref()
    }
}

impl<I: Iterator> Iterator for Lookahead<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let second = std::mem::replace(&mut self.buffer[1], self.iter.next());
        std::mem::replace(&mut self.buffer[0], second)
    }
}

struct SourceReader<'a> {
    iter: Lookahead<CharIndices<'a>>,
    pos: usize,
}

impl<'a> SourceReader<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            iter: Lookahead::new(input.char_indices()),
            pos: 0,
        }
    }

    fn next(&mut self) -> Option<char> {
        let (i, next) = self.iter.next()?;
        self.pos = i + next.len_utf8();
        Some(next)
    }

    fn peek(&self) -> Option<&char> {
        let (_, next) = self.iter.peek()?;
        Some(next)
    }

    fn peek_2(&self) -> Option<&char> {
        let (_, next) = self.iter.peek_2()?;
        Some(next)
    }

    fn pos(&self) -> usize {
        self.pos
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    reader: SourceReader<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            reader: SourceReader::new(input),
        }
    }

    pub fn next_token(&mut self) -> Result<Token<'a>, LexerError> {
        self.consume_whitespace();

        let Some(&peek_1) = self.reader.peek() else {
            return Ok(Token {
                kind: TokenKind::Eof,
                span: Span::new(0, 0),
            });
        };

        let token = match (peek_1, self.reader.peek_2()) {
            ('_', _) => self.lex_ident()?,
            _ if peek_1.is_ascii_alphabetic() => self.lex_ident()?,
            _ if peek_1.is_ascii_digit() => self.lex_numeric()?,
            ('"', _) => self.lex_string()?,
            ('+', _) => self.lex_single(TokenKind::Plus),
            ('-', _) => self.lex_single(TokenKind::Minus),
            ('*', _) => self.lex_single(TokenKind::Star),
            ('/', _) => self.lex_single(TokenKind::Slash),
            ('%', _) => self.lex_single(TokenKind::Percent),
            ('|', Some('|')) => self.lex_single(TokenKind::LogicalOr),
            ('&', Some('&')) => self.lex_single(TokenKind::LogicalAnd),
            ('|', Some('>')) => self.lex_single(TokenKind::Pipeline),
            ('!', Some('=')) => self.lex_single(TokenKind::NotEqual),
            ('!', _) => self.lex_single(TokenKind::Bang),
            ('=', Some('=')) => self.lex_single(TokenKind::Equal),
            ('=', _) => self.lex_single(TokenKind::Assign),
            ('>', Some('=')) => self.lex_single(TokenKind::GreaterEqual),
            ('>', _) => self.lex_single(TokenKind::Greater),
            ('<', Some('=')) => self.lex_single(TokenKind::LessEqual),
            ('<', _) => self.lex_single(TokenKind::Less),
            (';', _) => self.lex_single(TokenKind::Semicolon),
            (':', Some(':')) => self.lex_single(TokenKind::DoubleColon),
            (':', _) => self.lex_single(TokenKind::Colon),
            (',', _) => self.lex_single(TokenKind::Comma),
            ('.', Some('.')) => self.lex_triple_dot()?,
            ('.', _) => self.lex_single(TokenKind::Dot),
            ('(', _) => self.lex_single(TokenKind::LParen),
            (')', _) => self.lex_single(TokenKind::RParen),
            ('[', _) => self.lex_single(TokenKind::LBracket),
            (']', _) => self.lex_single(TokenKind::RBracket),
            ('{', _) => self.lex_single(TokenKind::LBrace),
            ('}', _) => self.lex_single(TokenKind::RBrace),
            _ => return Err(LexerError::UnknownCharacter),
        };

        Ok(token)
    }

    fn consume_whitespace(&mut self) {
        // TODO: add support for comments

        while let Some(ch) = self.reader.peek()
            && ch.is_ascii_whitespace()
        {
            self.reader.next();
        }
    }

    fn consume_while(&mut self, mut condition: impl FnMut(&char) -> bool) -> Option<Span> {
        let start = self.reader.pos();
        let mut end = None;

        while let Some(ch) = self.reader.peek() {
            if condition(ch) {
                self.reader.next();
                end.get_or_insert(self.reader.pos());
            } else {
                break;
            }
        }

        end.map(|e| Span::new(start, e))
    }

    fn lex_single(&mut self, kind: TokenKind<'a>) -> Token<'a> {
        let start = self.reader.pos();
        self.reader.next();

        Token {
            kind,
            span: Span::new(start, self.reader.pos()),
        }
    }

    fn lex_ident(&mut self) -> Result<Token<'a>, LexerError> {
        let span = self
            .consume_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
            .ok_or(LexerError::InternalError)?;
        let ident = &self.input[span.range()];

        // TODO: match keywords

        Ok(Token {
            kind: TokenKind::Ident(ident),
            span,
        })
    }

    fn lex_string(&mut self) -> Result<Token<'a>, LexerError> {
        let start = self.reader.pos();
        // TODO: add support for escaping `\"`
        let span = self.consume_while(|ch| *ch != '"');

        let content = span.map_or("", |s| &self.input[s.range()]);

        if let Some(ch) = self.reader.peek()
            && *ch == '"'
        {
            self.reader.next();
            return Ok(Token {
                kind: TokenKind::StringLit(content),
                span: Span::new(start, self.reader.pos()),
            });
        }

        Err(LexerError::UnclosedStringLit)
    }

    fn lex_numeric(&mut self) -> Result<Token<'a>, LexerError> {
        let int = self
            .consume_while(char::is_ascii_digit)
            .ok_or(LexerError::InternalError)?;

        let decimal = if let Some(ch) = self.reader.peek()
            && *ch == '.'
        {
            self.reader.next();
            let span = self
                .consume_while(char::is_ascii_digit)
                .ok_or(LexerError::MalformedFloat)?;
            Some(span)
        } else {
            None
        };

        let exponent = if let Some(ch) = self.reader.peek()
            && (*ch == 'e' || *ch == 'E')
        {
            self.reader.next();

            // handle negative exponent
            if let Some(ch) = self.reader.peek()
                && *ch == '-'
            {
                self.reader.next();
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

    fn lex_triple_dot(&mut self) -> Result<Token<'a>, LexerError> {
        let start = self.reader.pos();
        self.reader.next();
        self.reader.next();

        if let Some(ch) = self.reader.peek()
            && *ch == '.'
        {
            self.reader.next();
            return Ok(Token {
                kind: TokenKind::TripleDot,
                span: Span::new(start, self.reader.pos()),
            });
        }

        Err(LexerError::IncompleteToken)
    }
}
