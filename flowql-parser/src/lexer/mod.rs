use crate::error::{ParserResult, Span};
use crate::lexer::{
    cursor::SourceReader,
    error::{LexicalError, LexicalErrorKind},
    token::{Token, TokenKind},
};

mod cursor;
pub mod error;
pub mod token;

#[derive(Debug, Clone)]
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

    pub fn next_token(&mut self) -> ParserResult<Token<'a>> {
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
            ('+', _) => self.lex_chars(1, TokenKind::Plus),
            ('-', _) => self.lex_chars(1, TokenKind::Minus),
            ('*', _) => self.lex_chars(1, TokenKind::Star),
            ('/', _) => self.lex_chars(1, TokenKind::Slash),
            ('%', _) => self.lex_chars(1, TokenKind::Percent),
            ('|', Some('|')) => self.lex_chars(2, TokenKind::LogicalOr),
            ('|', Some('>')) => self.lex_chars(2, TokenKind::Pipeline),
            ('|', _) => Err(self.lex_garbage(1, LexicalErrorKind::IncompleteToken("||")))?,
            ('&', Some('&')) => self.lex_chars(2, TokenKind::LogicalAnd),
            ('&', _) => Err(self.lex_garbage(1, LexicalErrorKind::IncompleteToken("&&")))?,
            ('!', Some('=')) => self.lex_chars(2, TokenKind::NotEqual),
            ('!', _) => self.lex_chars(1, TokenKind::Bang),
            ('=', Some('=')) => self.lex_chars(2, TokenKind::Equal),
            ('=', _) => self.lex_chars(1, TokenKind::Assign),
            ('>', Some('=')) => self.lex_chars(2, TokenKind::GreaterEqual),
            ('>', _) => self.lex_chars(1, TokenKind::Greater),
            ('<', Some('=')) => self.lex_chars(2, TokenKind::LessEqual),
            ('<', _) => self.lex_chars(1, TokenKind::Less),
            (';', _) => self.lex_chars(1, TokenKind::Semicolon),
            (':', Some(':')) => self.lex_chars(2, TokenKind::DoubleColon),
            (':', _) => self.lex_chars(1, TokenKind::Colon),
            (',', _) => self.lex_chars(1, TokenKind::Comma),
            ('.', Some('.')) => self.lex_triple_dot()?,
            ('.', _) => self.lex_chars(1, TokenKind::Dot),
            ('(', _) => self.lex_chars(1, TokenKind::LParen),
            (')', _) => self.lex_chars(1, TokenKind::RParen),
            ('[', _) => self.lex_chars(1, TokenKind::LBracket),
            (']', _) => self.lex_chars(1, TokenKind::RBracket),
            ('{', _) => self.lex_chars(1, TokenKind::LBrace),
            ('}', _) => self.lex_chars(1, TokenKind::RBrace),
            _ => {
                let start = self.reader.pos();
                self.reader.next();

                Err(LexicalError {
                    kind: LexicalErrorKind::UnknownCharacter(peek_1),
                    span: Span::new(start, self.reader.pos()),
                })?
            }
        };

        Ok(token)
    }

    fn consume_whitespace(&mut self) -> Result<(), LexicalError> {
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
                            nested -= 1;
                            if nested == 0 {
                                break;
                            }
                        }
                        (None, _) => {
                            return Err(LexicalError {
                                kind: LexicalErrorKind::UnclosedComment,
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

    fn lex_garbage(&mut self, num: usize, err_kind: LexicalErrorKind) -> LexicalError {
        let start = self.reader.pos();
        for _ in 0..num {
            self.reader.next();
        }

        LexicalError {
            kind: err_kind,
            span: Span::new(start, self.reader.pos()),
        }
    }

    fn lex_chars(&mut self, num: usize, kind: TokenKind<'a>) -> Token<'a> {
        let start = self.reader.pos();
        for _ in 0..num {
            self.reader.next();
        }

        Token {
            kind,
            span: Span::new(start, self.reader.pos()),
        }
    }

    fn lex_ident(&mut self) -> Result<Token<'a>, LexicalError> {
        let span = self
            .consume_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
            .ok_or_else(|| LexicalError {
                kind: LexicalErrorKind::InternalError("expected identifier to have length >= 1"),
                span: Span::new(self.reader.pos(), self.reader.pos()),
            })?;
        let ident = &self.input[span.range()];

        let keyword = match ident {
            "if" => Some(TokenKind::If),
            "let" => Some(TokenKind::Let),
            "create" => Some(TokenKind::Create),
            "set" => Some(TokenKind::Set),
            "migrate" => Some(TokenKind::Migrate),
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

    fn lex_string(&mut self) -> Result<Token<'a>, LexicalError> {
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
                return Err(LexicalError {
                    kind: LexicalErrorKind::UnclosedStringLit,
                    span: Span::new(start, self.reader.pos()),
                });
            }
        }
    }

    fn lex_numeric(&mut self) -> Result<Token<'a>, LexicalError> {
        // TODO: support underscores in between digits

        let radix = match (self.reader.peek(), self.reader.peek_2()) {
            (Some('0'), Some('x')) => 16,
            (Some('0'), Some('o')) => 8,
            (Some('0'), Some('b')) => 2,
            _ => 10,
        };
        if radix != 10 {
            self.reader.next();
            self.reader.next();
        }

        let int = self
            .consume_while(|ch| ch.is_digit(radix))
            .ok_or_else(|| LexicalError {
                kind: LexicalErrorKind::InternalError("expected number to have length >= 1"),
                span: Span::new(self.reader.pos(), self.reader.pos()),
            })?;

        let fractional = if let Some(peek) = self.reader.peek()
            && *peek == '.'
            && let Some(peek_2) = self.reader.peek_2()
            && peek_2.is_ascii_digit()
        {
            self.reader.next();
            Some(self.consume_while(char::is_ascii_digit).unwrap())
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
                .ok_or_else(|| LexicalError {
                    kind: LexicalErrorKind::IncompleteFloatExponent,
                    span: Span::new(int.start, self.reader.pos()),
                })?;
            Some(span)
        } else {
            None
        };

        // number is an integer
        if fractional.is_none() && exponent.is_none() {
            let parsed =
                i64::from_str_radix(&self.input[int.range()], radix).map_err(|_| LexicalError {
                    kind: LexicalErrorKind::IntegerOverflow,
                    span: int,
                })?;

            return Ok(Token {
                kind: TokenKind::IntLit(parsed),
                span: int,
            });
        }

        let mut span = int;

        if let Some(frac) = fractional {
            span = span.merge(&frac);
        }
        if let Some(exp) = exponent {
            span = span.merge(&exp);
        }

        // error on non-decimal float literal
        if radix != 10 {
            return Err(LexicalError {
                kind: match radix {
                    16 => LexicalErrorKind::HexadecimalFloat,
                    8 => LexicalErrorKind::OctalFloat,
                    2 => LexicalErrorKind::BinaryFloat,
                    _ => unreachable!(),
                },
                span,
            });
        }

        Ok(Token {
            kind: TokenKind::FloatLit(self.input[span.range()].parse().map_err(|_| {
                LexicalError {
                    kind: LexicalErrorKind::InternalError("failed to parse float literal"),
                    span,
                }
            })?),
            span,
        })
    }

    fn lex_triple_dot(&mut self) -> Result<Token<'a>, LexicalError> {
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

        Err(LexicalError {
            kind: LexicalErrorKind::IncompleteToken("..."),
            span: Span::new(start, self.reader.pos()),
        })
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = ParserResult<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_token())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> ParserResult<Vec<Token<'_>>> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();

        loop {
            let tok = lexer.next_token()?;
            if tok.is_eof() {
                break;
            }
            tokens.push(tok);
        }

        Ok(tokens)
    }

    fn kinds<'a>(tokens: &[Token<'a>]) -> Vec<TokenKind<'a>> {
        tokens.iter().map(|tok| tok.kind).collect()
    }

    fn spans(tokens: &[Token]) -> Vec<Span> {
        tokens.iter().map(|tok| tok.span).collect()
    }

    #[test]
    fn lexes_ident() {
        let tokens = lex("abc123 _987 if XYZ_").unwrap();

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::Ident("abc123"),
                TokenKind::Ident("_987"),
                TokenKind::If,
                TokenKind::Ident("XYZ_"),
            ]
        );
        assert_eq!(
            spans(&tokens),
            vec![
                Span::new(0, 6),
                Span::new(7, 11),
                Span::new(12, 14),
                Span::new(15, 19),
            ]
        );
    }

    #[test]
    fn lexes_numeric() {
        let tokens = lex("9876 0x7FFA 0b11101110 1E+3 0.005 00040.550e-256").unwrap();

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::IntLit(9876),
                TokenKind::IntLit(0x7FFA),
                TokenKind::IntLit(0b11101110),
                TokenKind::FloatLit(1e3),
                TokenKind::FloatLit(0.005),
                TokenKind::FloatLit(40.55e-256),
            ]
        );

        assert_eq!(
            kinds(&lex("1.").unwrap()),
            vec![TokenKind::IntLit(1), TokenKind::Dot]
        );
        assert_eq!(
            kinds(&lex(".02").unwrap()),
            vec![TokenKind::Dot, TokenKind::IntLit(2)]
        );
        assert!(lex("4e").is_err());
        assert!(lex("0xFF.0").is_err());
        assert!(lex("9223372036854775808").is_err());
    }

    #[test]
    fn lexes_string() {
        let tokens = lex(r#""words  🟥 \" " "üéÆç""#).unwrap();

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::StringLit(r#"words  🟥 \" "#),
                TokenKind::StringLit("üéÆç")
            ]
        );

        assert!(lex(r#""unclosed "#).is_err());
    }

    #[test]
    fn lexes_operators() {
        let tokens = lex("+-*/%!&&|||>").unwrap();

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Percent,
                TokenKind::Bang,
                TokenKind::LogicalAnd,
                TokenKind::LogicalOr,
                TokenKind::Pipeline,
            ]
        );

        assert!(lex("|").is_err());
        assert!(lex("&").is_err());
    }

    #[test]
    fn lexes_comparison() {
        let tokens = lex("!!====>>=<=<").unwrap();

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::Bang,
                TokenKind::NotEqual,
                TokenKind::Equal,
                TokenKind::Assign,
                TokenKind::Greater,
                TokenKind::GreaterEqual,
                TokenKind::LessEqual,
                TokenKind::Less,
            ]
        );
        assert_eq!(
            spans(&tokens),
            vec![
                Span::new(0, 1),
                Span::new(1, 3),
                Span::new(3, 5),
                Span::new(5, 6),
                Span::new(6, 7),
                Span::new(7, 9),
                Span::new(9, 11),
                Span::new(11, 12),
            ]
        );
    }

    #[test]
    fn lexes_punctuation() {
        let tokens = lex("::;:,....([{)]}").unwrap();

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::DoubleColon,
                TokenKind::Semicolon,
                TokenKind::Colon,
                TokenKind::Comma,
                TokenKind::TripleDot,
                TokenKind::Dot,
                TokenKind::LParen,
                TokenKind::LBracket,
                TokenKind::LBrace,
                TokenKind::RParen,
                TokenKind::RBracket,
                TokenKind::RBrace,
            ]
        );

        assert!(lex("..").is_err());
    }

    #[test]
    fn lexes_whitespace() {
        assert!(lex(" \t\r\n").unwrap().is_empty());

        let comments = r#"
        // this is a comment
        /* block comment
           /* sub-comment */
        */
        "#;
        assert!(lex(comments).unwrap().is_empty());

        assert!(lex("/* ").is_err());
    }
}
