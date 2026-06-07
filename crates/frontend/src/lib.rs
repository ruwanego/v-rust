#![forbid(unsafe_code)]

pub mod lex;
pub mod parse;
pub mod sema;
pub mod source;
pub mod types;

use chumsky::{stream::Stream, Parser};
use lex::SpannedToken;
use parse::ast::Program;
use source::{empty_span, Span};

pub fn parse_tokens(tokens: Vec<SpannedToken>) -> Result<Program, String> {
    let eoi = tokens.last().map_or_else(|| empty_span(0), |(_, span)| empty_span(span.end));
    let stream = Stream::from_iter(eoi, tokens.into_iter());
    Parser::parse(&parse::parser(), stream).map_err(|errors| format_parse_errors(&errors))
}

fn format_parse_errors(errors: &[chumsky::error::Simple<lex::Token, Span>]) -> String {
    format!("Parser error: {errors:?}")
}
