use crate::source::{line_column, Span, Spanned};
use logos::Logos;
use std::path::Path;

pub type SpannedToken = Spanned<Token>;

#[derive(Logos, Debug, PartialEq, Eq, Hash, Clone)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(skip r"//[^\r\n]*")]
pub enum Token {
    #[token("fn")]
    Fn,

    #[token("mut")]
    Mut,

    #[token("module")]
    Module,

    #[token("import")]
    Import,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(",")]
    Comma,

    #[token(":=")]
    DeclAssign,

    #[token("==")]
    Eq,

    #[token("!=")]
    NotEq,

    #[token("<=")]
    LtEq,

    #[token(">=")]
    GtEq,

    #[token("<")]
    Lt,

    #[token(">")]
    Gt,

    #[token("=")]
    Assign,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    #[token("&&")]
    And,

    #[token("||")]
    Or,

    #[token("!")]
    Not,

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[regex(r#"'[^']*'"#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    StringLiteral(String),

    #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
    IntLiteral(i64),
}

pub fn tokenize(source: &str, input: &Path) -> Result<Vec<SpannedToken>, String> {
    Token::lexer(source)
        .spanned()
        .map(|(token, span)| {
            token.map(|token| (token, span.clone())).map_err(|()| {
                let (line, column) = line_column(source, span.start);
                let unexpected = unexpected_character(source, &span);
                format!(
                    "Lex error at {}:{line}:{column}: unexpected character {}",
                    input.display(),
                    format_character(unexpected)
                )
            })
        })
        .collect()
}

fn unexpected_character(source: &str, span: &Span) -> char {
    source.get(span.clone()).and_then(|slice| slice.chars().next()).unwrap_or('\0')
}

fn format_character(character: char) -> String {
    match character {
        '\0' => "end of file".to_string(),
        '\n' => "'\\n'".to_string(),
        '\r' => "'\\r'".to_string(),
        '\t' => "'\\t'".to_string(),
        other => format!("'{other}'"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let mut lex = Token::lexer("fn main() { mut x := 5 \n x = 7 \n println(x) }");
        assert_eq!(lex.next(), Some(Ok(Token::Fn)));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("main".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::LParen)));
        assert_eq!(lex.next(), Some(Ok(Token::RParen)));
        assert_eq!(lex.next(), Some(Ok(Token::LBrace)));
        assert_eq!(lex.next(), Some(Ok(Token::Mut)));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("x".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::DeclAssign)));
        assert_eq!(lex.next(), Some(Ok(Token::IntLiteral(5))));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("x".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Assign)));
        assert_eq!(lex.next(), Some(Ok(Token::IntLiteral(7))));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("println".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::LParen)));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("x".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::RParen)));
        assert_eq!(lex.next(), Some(Ok(Token::RBrace)));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn skips_line_comments() {
        let tokens: Vec<_> = Token::lexer("// ignored\nfn main() {\n    // ignored too\n}")
            .map(Result::unwrap)
            .collect();

        assert_eq!(
            tokens,
            vec![
                Token::Fn,
                Token::Identifier("main".to_string()),
                Token::LParen,
                Token::RParen,
                Token::LBrace,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn tokenize_retains_byte_spans() {
        let tokens = tokenize("fn main() {}", Path::new("<test>")).unwrap();

        assert_eq!(tokens[0], (Token::Fn, 0..2));
        assert_eq!(tokens[1], (Token::Identifier("main".to_string()), 3..7));
        assert_eq!(tokens[2], (Token::LParen, 7..8));
    }

    #[test]
    fn tokenizes_module_keyword() {
        let tokens = tokenize("module main", Path::new("<test>")).unwrap();

        assert_eq!(tokens[0], (Token::Module, 0..6));
        assert_eq!(tokens[1], (Token::Identifier("main".to_string()), 7..11));
    }

    #[test]
    fn tokenizes_import_keyword() {
        let tokens = tokenize("import os", Path::new("<test>")).unwrap();

        assert_eq!(tokens[0], (Token::Import, 0..6));
        assert_eq!(tokens[1], (Token::Identifier("os".to_string()), 7..9));
    }
}
