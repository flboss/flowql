use crate::{
    error::Span,
    lexer::{
        cursor::SourceReader,
        error::{LexerError, LexerErrorKind},
        token::{Token, TokenKind},
    },
};

mod cursor;
pub mod error;
mod token;

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
                span: Span::new(self.reader.pos(), self.reader.pos()),
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
            ('|', Some('>')) => self.lex_single(TokenKind::Pipeline),
            // TODO: add helper method for incomplete tokens
            ('|', _) => {
                return Err(LexerError {
                    kind: LexerErrorKind::IncompleteToken("||"),
                    span: Span::new(self.reader.pos(), self.reader.pos()),
                });
            }
            ('&', Some('&')) => self.lex_single(TokenKind::LogicalAnd),
            ('&', _) => {
                return Err(LexerError {
                    kind: LexerErrorKind::IncompleteToken("&&"),
                    span: Span::new(self.reader.pos(), self.reader.pos()),
                });
            }
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
            _ => {
                let start = self.reader.pos();
                self.reader.next();

                return Err(LexerError {
                    kind: LexerErrorKind::UnknownCharacter(peek_1),
                    span: Span::new(start, self.reader.pos()),
                });
            }
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

        while let Some(ch) = self.reader.peek() {
            if condition(ch) {
                self.reader.next();
            } else {
                break;
            }
        }

        if start != self.reader.pos() {
            Some(Span::new(start, self.reader.pos()))
        } else {
            None
        }
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
            .ok_or_else(|| LexerError {
                kind: LexerErrorKind::InternalError("expected identifier to have length >= 1"),
                span: Span::new(self.reader.pos(), self.reader.pos()),
            })?;
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

        Err(LexerError {
            kind: LexerErrorKind::UnclosedStringLit,
            span: Span::new(start, self.reader.pos()),
        })
    }

    fn lex_numeric(&mut self) -> Result<Token<'a>, LexerError> {
        let int = self
            .consume_while(char::is_ascii_digit)
            .ok_or_else(|| LexerError {
                kind: LexerErrorKind::InternalError("expected number to have length >= 1"),
                span: Span::new(self.reader.pos(), self.reader.pos()),
            })?;

        let decimal = if let Some(ch) = self.reader.peek()
            && *ch == '.'
        {
            self.reader.next();
            let span = self
                .consume_while(char::is_ascii_digit)
                .ok_or_else(|| LexerError {
                    kind: LexerErrorKind::MalformedFloat,
                    span: Span::new(self.reader.pos(), self.reader.pos()),
                })?;
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
                .ok_or_else(|| LexerError {
                    kind: LexerErrorKind::MalformedFloat,
                    span: Span::new(self.reader.pos(), self.reader.pos()),
                })?;
            Some(span)
        } else {
            None
        };

        // TODO: validate that number is not out of range before parsing

        // number is an integer
        if decimal.is_none() && exponent.is_none() {
            return Ok(Token {
                kind: TokenKind::IntLit(self.input[int.range()].parse().map_err(|_| {
                    LexerError {
                        kind: LexerErrorKind::InternalError(
                            "expected integer parsing to be successful",
                        ),
                        span: int,
                    }
                })?),
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
            kind: TokenKind::FloatLit(self.input[span.range()].parse().map_err(|_| {
                LexerError {
                    kind: LexerErrorKind::InternalError(
                        "expected integer parsing to be successful",
                    ),
                    span,
                }
            })?),
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

        Err(LexerError {
            kind: LexerErrorKind::IncompleteToken("..."),
            span: Span::new(start, self.reader.pos()),
        })
    }
}
