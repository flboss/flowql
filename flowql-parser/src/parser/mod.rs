use std::iter::Peekable;

use crate::error::ParserResult;
use crate::lexer::Lexer;
use crate::lexer::token::Token;
use crate::parser::ast::{Program, Statement};

pub mod ast;
pub mod error;

pub struct Parser<'a, I>
where
    I: Iterator<Item = ParserResult<Token<'a>>>,
{
    tokens: Peekable<I>,
    ast: Vec<Statement<'a>>,
}

impl<'a> Parser<'a, Lexer<'a>> {
    pub fn new(input: &'a str) -> Parser<'a, Lexer<'a>> {
        Parser {
            tokens: Lexer::new(input).peekable(),
            ast: Vec::new(),
        }
    }

    pub fn parse_program(self) -> ParserResult<&'a Program<'a>> {
        todo!()
    }

    fn parse_statement(&self) -> ParserResult<&'a Statement<'a>> {
        todo!()
    }
}
