use logos::Logos;
use std::ops::Range;
use std::path::Path;

#[derive(Logos, Debug, PartialEq, Eq, Hash, Clone)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(skip r"//[^\r\n]*")]
pub enum Token {
    #[token("fn")]
    Fn,

    #[token("mut")]
    Mut,

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

pub fn tokenize(source: &str, input: &Path) -> Result<Vec<Token>, String> {
    Token::lexer(source)
        .spanned()
        .map(|(token, span)| {
            token.map_err(|()| {
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

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut line_start = 0;

    for (index, character) in source.char_indices() {
        if index >= offset {
            break;
        }

        if character == '\n' {
            line += 1;
            line_start = index + character.len_utf8();
        }
    }

    let column = source
        .get(line_start..offset)
        .map_or(1, |line| line.chars().count() + 1);
    (line, column)
}

fn unexpected_character(source: &str, span: &Range<usize>) -> char {
    source
        .get(span.clone())
        .and_then(|slice| slice.chars().next())
        .unwrap_or('\0')
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
}
