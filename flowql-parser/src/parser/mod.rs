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
    pub ast: Program,
}

impl<'a> Parser<'a, Lexer<'a>> {
    pub fn new(input: &'a str) -> Parser<'a, Lexer<'a>> {
        Parser {
            tokens: Lexer::new(input).peekable(),
            ast: Program::new(),
        }
    }

    pub fn parse_program(&mut self) -> ParserResult<&Program> {
        todo!()
    }

    fn parse_statement(&mut self) -> ParserResult<&Statement> {
        todo!()
    }
}
