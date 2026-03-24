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
        self.consume_whitespace()?;

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

    fn consume_whitespace(&mut self) -> Result<(), LexerError> {
        let start = self.reader.pos();
        let mut nested = 0;

        loop {
            while let Some(ch) = self.reader.peek()
                && ch.is_ascii_whitespace()
            {
                self.reader.next();
            }

            match (self.reader.peek(), self.reader.peek_2()) {
                // line comment
                (Some('/'), Some('/')) => {
                    self.consume_while(|ch| *ch != '\n');
                    self.reader.next();
                }
                // block comment (with nesting)
                (Some('/'), Some('*')) => loop {
                    self.consume_while(|ch| *ch != '/' && *ch != '*');
                    match (self.reader.peek(), self.reader.peek_2()) {
                        (Some('/'), Some('*')) => {
                            self.reader.next();
                            self.reader.next();
                            nested += 1;
                        }
                        (Some('*'), Some('/')) => {
                            self.reader.next();
                            self.reader.next();
                            if nested == 0 {
                                break;
                            }
                            nested -= 1;
                        }
                        (None, _) => {
                            return Err(LexerError {
                                kind: LexerErrorKind::UnclosedComment,
                                span: Span::new(start, self.reader.pos()),
                            });
                        }
                        _ => {}
                    }
                },
                _ => break,
            }
        }
        Ok(())
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

        let keyword = match ident {
            "if" => Some(TokenKind::If),
            "let" => Some(TokenKind::Let),
            "store" => Some(TokenKind::Store),
            "drop" => Some(TokenKind::Drop),
            _ => None,
        };

        if let Some(kw) = keyword {
            return Ok(Token { kind: kw, span });
        }

        Ok(Token {
            kind: TokenKind::Ident(ident),
            span,
        })
    }

    fn lex_string(&mut self) -> Result<Token<'a>, LexerError> {
        let start = self.reader.pos();
        self.reader.next();
        let start_content = self.reader.pos();

        loop {
            self.consume_while(|ch| *ch != '"' && *ch != '\\');
            if let Some(ch) = self.reader.peek() {
                match (ch, self.reader.peek_2()) {
                    ('"', _) => {
                        let content = &self.input[start_content..self.reader.pos()];
                        self.reader.next();
                        return Ok(Token {
                            kind: TokenKind::StringLit(content),
                            span: Span::new(start, self.reader.pos()),
                        });
                    }
                    ('\\', Some('"')) => {
                        self.reader.next();
                        self.reader.next();
                    }
                    // TODO: replace escape sequence
                    // TODO: implement other escaping syntax (`\\`, `\n`, `\t`, `\r`, `\x00` (ascii), `\u0000` (unicode))
                    ('\\', _) => {
                        self.reader.next();
                    }
                    _ => {}
                }
            } else {
                return Err(LexerError {
                    kind: LexerErrorKind::UnclosedStringLit,
                    span: Span::new(start, self.reader.pos()),
                });
            }
        }
    }

    fn lex_numeric(&mut self) -> Result<Token<'a>, LexerError> {
        // TODO: support underscores in between digits
        // TODO: hex (`0x`), binary (`0b`), octal (`0o`) literals

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
                    kind: LexerErrorKind::IncompleteFloatDecimal,
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

            // handle exponent sign
            if let Some(ch) = self.reader.peek()
                && (*ch == '-' || *ch == '+')
            {
                self.reader.next();
            }

            let span = self
                .consume_while(char::is_ascii_digit)
                .ok_or_else(|| LexerError {
                    kind: LexerErrorKind::IncompleteFloatExponent,
                    span: Span::new(self.reader.pos(), self.reader.pos()),
                })?;
            Some(span)
        } else {
            None
        };

        // number is an integer
        if decimal.is_none() && exponent.is_none() {
            let parsed = self.input[int.range()].parse().map_err(|_| LexerError {
                kind: LexerErrorKind::IntegerOverflow,
                span: int,
            })?;

            return Ok(Token {
                kind: TokenKind::IntLit(parsed),
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
