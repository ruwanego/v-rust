#![forbid(unsafe_code)]

pub mod lex;
pub mod parse;
pub mod sema;

use chumsky::Parser;
use parse::ast::Program;

pub fn parse_tokens(tokens: Vec<lex::Token>) -> Result<Program, String> {
    Parser::parse(&parse::parser(), tokens).map_err(|errors| format!("Parser error: {errors:?}"))
}
