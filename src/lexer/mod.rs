use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let mut lex = Token::lexer("fn main() {}");
        assert_eq!(lex.next(), Some(Ok(Token::Fn)));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("main".to_string())))); // main
        assert_eq!(lex.slice(), "main");
        assert_eq!(lex.next(), Some(Ok(Token::LParen)));
        assert_eq!(lex.next(), Some(Ok(Token::RParen)));
        assert_eq!(lex.next(), Some(Ok(Token::LBrace)));
        assert_eq!(lex.next(), Some(Ok(Token::RBrace)));
        assert_eq!(lex.next(), None);
    }
}
