pub mod ast;

use crate::lexer::Token;
use ast::{FunctionDecl, Program};
use chumsky::prelude::*;

pub fn parser() -> impl Parser<Token, Program, Error = Simple<Token>> {
    let identifier = select! {
        Token::Identifier(name) => name,
    };

    let function_decl = just(Token::Fn)
        .ignore_then(identifier)
        .then_ignore(just(Token::LParen))
        .then_ignore(just(Token::RParen))
        .then_ignore(just(Token::LBrace))
        .then_ignore(just(Token::RBrace))
        .map(|name| FunctionDecl { name });

    function_decl
        .repeated()
        .map(|functions| Program { functions })
        .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    #[test]
    fn test_parser() {
        let tokens: Vec<_> = Token::lexer("fn main() {}").map(|res| res.unwrap()).collect();
        let program = parser().parse(tokens).unwrap();
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "main");
    }
}
