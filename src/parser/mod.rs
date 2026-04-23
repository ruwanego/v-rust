pub mod ast;

use crate::lexer::Token;
use ast::{Expr, FunctionDecl, Program};
use chumsky::prelude::*;

pub fn parser() -> impl Parser<Token, Program, Error = Simple<Token>> {
    let identifier = select! {
        Token::Identifier(name) => name,
    };

    let expr = recursive(|expr| {
        let string_lit = select! {
            Token::StringLiteral(s) => Expr::StringLiteral(s),
        };
        let int_lit = select! {
            Token::IntLiteral(i) => Expr::IntLiteral(i),
        };

        let args = expr
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::LParen), just(Token::RParen));

        let func_call = identifier
            .then(args)
            .map(|(name, args)| Expr::FunctionCall { name, args });

        choice((func_call, string_lit, int_lit))
    });

    let block = expr
        .repeated()
        .delimited_by(just(Token::LBrace), just(Token::RBrace));

    let function_decl = just(Token::Fn)
        .ignore_then(identifier)
        .then_ignore(just(Token::LParen))
        .then_ignore(just(Token::RParen))
        .then(block)
        .map(|(name, body)| FunctionDecl { name, body });

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
        let tokens: Vec<_> = Token::lexer("fn main() { println('hello') 42 }")
            .map(|res| res.unwrap())
            .collect();
        let program = parser().parse(tokens).unwrap();
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "main");
        assert_eq!(program.functions[0].body.len(), 2);
        
        match &program.functions[0].body[0] {
            Expr::FunctionCall { name, args } => {
                assert_eq!(name, "println");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], Expr::StringLiteral("hello".to_string()));
            }
            _ => panic!("Expected FunctionCall"),
        }
        
        assert_eq!(program.functions[0].body[1], Expr::IntLiteral(42));
    }
}
