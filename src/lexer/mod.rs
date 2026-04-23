use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq, Hash, Clone)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("fn")]
    Fn,

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

    #[regex(r#"'[^']*'"#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    StringLiteral(String),

    #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
    IntLiteral(i64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let mut lex = Token::lexer("fn main() { println('hello') 42 }");
        assert_eq!(lex.next(), Some(Ok(Token::Fn)));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("main".to_string()))));
        assert_eq!(lex.slice(), "main");
        assert_eq!(lex.next(), Some(Ok(Token::LParen)));
        assert_eq!(lex.next(), Some(Ok(Token::RParen)));
        assert_eq!(lex.next(), Some(Ok(Token::LBrace)));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("println".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::LParen)));
        assert_eq!(lex.next(), Some(Ok(Token::StringLiteral("hello".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::RParen)));
        assert_eq!(lex.next(), Some(Ok(Token::IntLiteral(42))));
        assert_eq!(lex.next(), Some(Ok(Token::RBrace)));
        assert_eq!(lex.next(), None);
    }
}
