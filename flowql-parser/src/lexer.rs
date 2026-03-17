use std::str::CharIndices;

pub struct Token<'a> {
    kind: TokenKind<'a>,
    span: Span,
}

pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
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
    Comma,
    Dot,
    TripleDot,

    // delimiters
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,

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
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    Assign,

    // keywords
    If,
    Let,
    Store,
    Drop,

    // TODO: design error architecture
    // error
    Invalid(&'a str),
}

pub enum LexerError {
    UnknownCharacter,
    IncompleteToken,
    UnclosedComment,
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

    pub fn next_token(&mut self) -> Token<'a> {
        let Some((_, ch)) = self.chars.peek() else {
            return Token {
                kind: TokenKind::Eof,
                span: Span::new(0, 0),
            };
        };

        let token = match ch {
            '_' => self.lex_ident(),
            _ if ch.is_ascii_alphabetic() => self.lex_ident(),
            '-' => self.consume_single(TokenKind::Minus),
            _ if ch.is_ascii_digit() => self.lex_numeric(),
            '+' => self.consume_single(TokenKind::Plus),
            '*' => self.consume_single(TokenKind::Star),
            '/' => self.lex_slash(),
            '%' => self.consume_single(TokenKind::Percent),
            '!' => self.lex_bang(),
            '=' => self.lex_equal(),
            '>' => self.lex_greater(),
            '<' => self.lex_less(),
            ';' => self.consume_single(TokenKind::Semicolon),
            ':' => self.consume_single(TokenKind::Colon),
            ',' => self.consume_single(TokenKind::Comma),
            '.' => self.lex_dot(),
            '(' => self.consume_single(TokenKind::LeftParen),
            ')' => self.consume_single(TokenKind::RightParen),
            '[' => self.consume_single(TokenKind::LeftBracket),
            ']' => self.consume_single(TokenKind::RightBracket),
            '{' => self.consume_single(TokenKind::LeftBrace),
            '}' => self.consume_single(TokenKind::RightBrace),
            other => todo!(),
        };

        // TODO: skip whitespace/comments

        todo!()
    }

    fn consume_single(&mut self, kind: TokenKind<'a>) -> Token<'a> {
        let (start, ch) = self.chars.next().unwrap();

        Token {
            kind,
            span: Span::new(start, start + ch.len_utf8()),
        }
    }

    fn lex_ident(&mut self) -> Token<'a> {
        todo!()
    }

    fn lex_numeric(&mut self) -> Token<'a> {
        todo!()
    }

    fn lex_slash(&mut self) -> Token<'a> {
        todo!()
    }

    fn lex_dot(&mut self) -> Token<'a> {
        todo!()
    }

    fn lex_bang(&mut self) -> Token<'a> {
        todo!()
    }

    fn lex_equal(&mut self) -> Token<'a> {
        todo!()
    }

    fn lex_greater(&mut self) -> Token<'a> {
        todo!()
    }

    fn lex_less(&mut self) -> Token<'a> {
        todo!()
    }
}
