pub mod ast;

use crate::lex::Token;
use ast::{Expr, FunctionDecl, Program, Stmt};
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
        let bool_lit = choice((
            just(Token::True).to(Expr::BoolLiteral(true)),
            just(Token::False).to(Expr::BoolLiteral(false)),
        ));
        let ident_expr = identifier.clone().map(Expr::Identifier);

        let args = expr.clone()
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::LParen), just(Token::RParen));

        let func_call = identifier.clone()
            .then(args)
            .map(|(name, args)| Expr::FunctionCall { name, args });

        let atom = choice((
            func_call,
            string_lit,
            int_lit,
            bool_lit,
            ident_expr,
            expr.clone().delimited_by(just(Token::LParen), just(Token::RParen)),
        ));

        let unary = choice((
            just(Token::Minus).to(ast::UnaryOp::Minus),
            just(Token::Not).to(ast::UnaryOp::Not),
        ))
        .repeated()
        .then(atom)
        .foldr(|op, rhs| Expr::Unary {
            op,
            expr: Box::new(rhs),
        });

        let factor = unary.clone()
            .then(choice((
                just(Token::Star).to(ast::BinaryOp::Mul),
                just(Token::Slash).to(ast::BinaryOp::Div),
                just(Token::Percent).to(ast::BinaryOp::Mod),
            )).then(unary).repeated())
            .foldl(|lhs, (op, rhs)| Expr::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            });

        let term = factor.clone()
            .then(choice((
                just(Token::Plus).to(ast::BinaryOp::Add),
                just(Token::Minus).to(ast::BinaryOp::Sub),
            )).then(factor).repeated())
            .foldl(|lhs, (op, rhs)| Expr::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            });

        let comparison = term.clone()
            .then(choice((
                just(Token::Eq).to(ast::BinaryOp::Eq),
                just(Token::NotEq).to(ast::BinaryOp::NotEq),
                just(Token::LtEq).to(ast::BinaryOp::LtEq),
                just(Token::GtEq).to(ast::BinaryOp::GtEq),
                just(Token::Lt).to(ast::BinaryOp::Lt),
                just(Token::Gt).to(ast::BinaryOp::Gt),
            )).then(term).repeated())
            .foldl(|lhs, (op, rhs)| Expr::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            });

        let logical_and = comparison.clone()
            .then(just(Token::And).to(ast::BinaryOp::And).then(comparison).repeated())
            .foldl(|lhs, (op, rhs)| Expr::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            });

        let logical_or = logical_and.clone()
            .then(just(Token::Or).to(ast::BinaryOp::Or).then(logical_and).repeated())
            .foldl(|lhs, (op, rhs)| Expr::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            });

        logical_or
    });

    let stmt = choice((
        just(Token::Mut).or_not()
            .then(identifier.clone())
            .then_ignore(just(Token::DeclAssign))
            .then(expr.clone())
            .map(|((mut_token, name), expr)| Stmt::VarDecl {
                name,
                is_mut: mut_token.is_some(),
                expr,
            }),
        identifier.clone()
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .map(|(name, expr)| Stmt::Assign { name, expr }),
        expr.clone().map(Stmt::ExprStmt),
    ));

    let block = stmt
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
        let tokens: Vec<_> = Token::lexer("fn main() { mut x := 5 \n x = 7 \n println(x) }")
            .map(|res| res.unwrap())
            .collect();
        let program = parser().parse(tokens).unwrap();
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "main");
        assert_eq!(program.functions[0].body.len(), 3);
        
        assert_eq!(
            program.functions[0].body[0],
            Stmt::VarDecl {
                name: "x".to_string(),
                is_mut: true,
                expr: Expr::IntLiteral(5),
            }
        );

        assert_eq!(
            program.functions[0].body[1],
            Stmt::Assign {
                name: "x".to_string(),
                expr: Expr::IntLiteral(7),
            }
        );

        match &program.functions[0].body[2] {
            Stmt::ExprStmt(Expr::FunctionCall { name, args }) => {
                assert_eq!(name, "println");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], Expr::Identifier("x".to_string()));
            }
            _ => panic!("Expected ExprStmt(FunctionCall)"),
        }
    }
}
