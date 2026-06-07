#![forbid(unsafe_code)]

pub mod lex;
pub mod parse;
pub mod sema;
pub mod source;
pub mod types;

use chumsky::{stream::Stream, Parser};
use lex::SpannedToken;
use parse::ast::Program;
use source::{empty_span, line_column, Span};
use std::fmt::Write as _;
use std::path::Path;

type ParseError = chumsky::error::Simple<lex::Token, Span>;

pub fn parse_tokens(
    source: &str,
    input: &Path,
    tokens: Vec<SpannedToken>,
) -> Result<Program, String> {
    parse_spanned_tokens(tokens).map_err(|errors| format_parse_errors(source, input, &errors))
}

fn parse_spanned_tokens(tokens: Vec<SpannedToken>) -> Result<Program, Vec<ParseError>> {
    let eoi = tokens.last().map_or_else(|| empty_span(0), |(_, span)| empty_span(span.end));
    let stream = Stream::from_iter(eoi, tokens.into_iter());
    Parser::parse(&parse::parser(), stream)
}

fn format_parse_errors(source: &str, input: &Path, errors: &[ParseError]) -> String {
    let mut message = String::new();

    for error in errors {
        let span = error.span();
        let (line, column) = line_column(source, span.start);
        writeln!(
            &mut message,
            "Parse error at {}:{line}:{column}: {}",
            input.display(),
            format_parse_error(error)
        )
        .expect("writing to a string cannot fail");
    }

    message
}

fn format_parse_error(error: &ParseError) -> String {
    let found = error.found().map_or_else(|| "end of input".to_string(), format_found_token);
    let expected = format_expected_tokens(error.expected());

    if expected.is_empty() {
        format!("unexpected {found}")
    } else {
        format!("expected {expected}, found {found}")
    }
}

fn format_expected_tokens<'a>(expected: impl Iterator<Item = &'a Option<lex::Token>>) -> String {
    let mut expected = expected
        .map(|token| {
            token.as_ref().map_or_else(|| "end of input".to_string(), format_expected_token)
        })
        .collect::<Vec<_>>();
    expected.sort();
    expected.dedup();

    match expected.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [head @ .., tail] => format!("{} or {tail}", head.join(", ")),
    }
}

fn format_expected_token(token: &lex::Token) -> String {
    match token {
        lex::Token::Fn => "`fn`".to_string(),
        lex::Token::Mut => "`mut`".to_string(),
        lex::Token::Identifier(_) => "identifier".to_string(),
        lex::Token::LParen => "`(`".to_string(),
        lex::Token::RParen => "`)`".to_string(),
        lex::Token::LBrace => "`{`".to_string(),
        lex::Token::RBrace => "`}`".to_string(),
        lex::Token::Comma => "`,`".to_string(),
        lex::Token::DeclAssign => "`:=`".to_string(),
        lex::Token::Eq => "`==`".to_string(),
        lex::Token::NotEq => "`!=`".to_string(),
        lex::Token::LtEq => "`<=`".to_string(),
        lex::Token::GtEq => "`>=`".to_string(),
        lex::Token::Lt => "`<`".to_string(),
        lex::Token::Gt => "`>`".to_string(),
        lex::Token::Assign => "`=`".to_string(),
        lex::Token::Plus => "`+`".to_string(),
        lex::Token::Minus => "`-`".to_string(),
        lex::Token::Star => "`*`".to_string(),
        lex::Token::Slash => "`/`".to_string(),
        lex::Token::Percent => "`%`".to_string(),
        lex::Token::And => "`&&`".to_string(),
        lex::Token::Or => "`||`".to_string(),
        lex::Token::Not => "`!`".to_string(),
        lex::Token::True => "`true`".to_string(),
        lex::Token::False => "`false`".to_string(),
        lex::Token::StringLiteral(_) => "string literal".to_string(),
        lex::Token::IntLiteral(_) => "integer literal".to_string(),
    }
}

fn format_found_token(token: &lex::Token) -> String {
    match token {
        lex::Token::Identifier(name) => format!("identifier `{name}`"),
        lex::Token::StringLiteral(value) => format!("string literal `{value}`"),
        lex::Token::IntLiteral(value) => format!("integer literal `{value}`"),
        other => format_expected_token(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_reports_token_location() {
        let source = "fn main() {\n    value :=\n}";
        let tokens = lex::tokenize(source, Path::new("<test>")).unwrap();

        let error = parse_tokens(source, Path::new("<test>"), tokens).unwrap_err();

        assert!(error.contains("Parse error at <test>:2:11:"), "{error}");
        assert!(error.contains("found `:=`"), "{error}");
        assert!(!error.contains("Parser error:"), "{error}");
    }

    #[test]
    fn parse_error_reports_end_of_input_location() {
        let source = "fn main() {\n    value := 1";
        let tokens = lex::tokenize(source, Path::new("<test>")).unwrap();

        let error = parse_tokens(source, Path::new("<test>"), tokens).unwrap_err();

        assert!(error.contains("Parse error at <test>:2:15:"), "{error}");
        assert!(error.contains("end of input"), "{error}");
    }
}
