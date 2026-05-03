use crate::error::ParserResult;
use crate::lexer::Lexer;
use crate::parser::ast::Statement;

pub mod ast;
pub mod error;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    ast: Vec<Statement<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Parser<'a> {
        Parser {
            lexer: Lexer::new(input),
            ast: Vec::new(),
        }
    }

    pub fn parse_statement(&self) -> ParserResult<&Statement<'a>> {
        todo!()
    }

    pub fn parse(&self) -> ParserResult<&Statement<'a>> {
        todo!()
    }
}
