pub mod ast;

use crate::lex::Token;
use crate::source::Span;
use ast::{Expr, ExprKind, FunctionDecl, ModuleDecl, Program, Stmt, StmtKind};
use chumsky::prelude::*;

#[must_use]
pub fn parser() -> impl Parser<Token, Program, Error = Simple<Token, Span>> {
    let identifier = select! {
        Token::Identifier(name) => name,
    }
    .map_with_span(|name, span: Span| (name, span));

    let expr = recursive(|expr| {
        let string_lit = select! {
            Token::StringLiteral(s) => s,
        }
        .map_with_span(|value, span: Span| Expr::new(ExprKind::StringLiteral(value), span));

        let int_lit = select! {
            Token::IntLiteral(i) => i,
        }
        .map_with_span(|value, span: Span| Expr::new(ExprKind::IntLiteral(value), span));

        let bool_lit = choice((
            just(Token::True).to(ExprKind::BoolLiteral(true)),
            just(Token::False).to(ExprKind::BoolLiteral(false)),
        ))
        .map_with_span(|kind, span: Span| Expr::new(kind, span));

        let ident_expr = identifier.map(|(name, span)| Expr::new(ExprKind::Identifier(name), span));

        let args = expr
            .clone()
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::LParen), just(Token::RParen));

        let func_call =
            identifier.then(args).map_with_span(|((name, _name_span), args), span: Span| {
                Expr::new(ExprKind::FunctionCall { name, args }, span)
            });

        let atom = choice((
            func_call,
            string_lit,
            int_lit,
            bool_lit,
            ident_expr,
            expr.clone().delimited_by(just(Token::LParen), just(Token::RParen)),
        ));

        let unary_op = choice((
            just(Token::Minus).to(ast::UnaryOp::Minus),
            just(Token::Not).to(ast::UnaryOp::Not),
        ));

        let unary = unary_op.repeated().then(atom).map_with_span(|(ops, atom), span: Span| {
            ops.into_iter().rev().fold(atom, |expr, op| {
                Expr::new(ExprKind::Unary { op, expr: Box::new(expr) }, span.clone())
            })
        });

        let factor = unary
            .clone()
            .then(
                choice((
                    just(Token::Star).to(ast::BinaryOp::Mul),
                    just(Token::Slash).to(ast::BinaryOp::Div),
                    just(Token::Percent).to(ast::BinaryOp::Mod),
                ))
                .then(unary)
                .repeated(),
            )
            .foldl(binary_expr);

        let term = factor
            .clone()
            .then(
                choice((
                    just(Token::Plus).to(ast::BinaryOp::Add),
                    just(Token::Minus).to(ast::BinaryOp::Sub),
                ))
                .then(factor)
                .repeated(),
            )
            .foldl(binary_expr);

        let comparison = term
            .clone()
            .then(
                choice((
                    just(Token::Eq).to(ast::BinaryOp::Eq),
                    just(Token::NotEq).to(ast::BinaryOp::NotEq),
                    just(Token::LtEq).to(ast::BinaryOp::LtEq),
                    just(Token::GtEq).to(ast::BinaryOp::GtEq),
                    just(Token::Lt).to(ast::BinaryOp::Lt),
                    just(Token::Gt).to(ast::BinaryOp::Gt),
                ))
                .then(term)
                .repeated(),
            )
            .foldl(binary_expr);

        let logical_and = comparison
            .clone()
            .then(just(Token::And).to(ast::BinaryOp::And).then(comparison).repeated())
            .foldl(binary_expr);

        logical_and
            .clone()
            .then(just(Token::Or).to(ast::BinaryOp::Or).then(logical_and).repeated())
            .foldl(binary_expr)
    });

    let stmt = choice((
        just(Token::Mut)
            .or_not()
            .then(identifier)
            .then_ignore(just(Token::DeclAssign))
            .then(expr.clone())
            .map_with_span(|((mut_token, (name, name_span)), expr), span: Span| {
                Stmt::new(
                    StmtKind::VarDecl { name, name_span, is_mut: mut_token.is_some(), expr },
                    span,
                )
            }),
        identifier.then_ignore(just(Token::Assign)).then(expr.clone()).map_with_span(
            |((name, name_span), expr), span: Span| {
                Stmt::new(StmtKind::Assign { name, name_span, expr }, span)
            },
        ),
        expr.clone().map_with_span(|expr, span: Span| Stmt::new(StmtKind::ExprStmt(expr), span)),
    ));

    let block = stmt.repeated().delimited_by(just(Token::LBrace), just(Token::RBrace));

    let function_decl = just(Token::Fn)
        .ignore_then(identifier)
        .then_ignore(just(Token::LParen))
        .then_ignore(just(Token::RParen))
        .then(block)
        .map_with_span(|((name, name_span), body), span: Span| FunctionDecl {
            name,
            name_span,
            body,
            span,
        });

    let module_decl = just(Token::Module)
        .ignore_then(identifier)
        .map_with_span(|(name, name_span), span: Span| ModuleDecl { name, name_span, span });

    module_decl
        .or_not()
        .then(function_decl.repeated())
        .map_with_span(|(module, functions), span: Span| Program { module, functions, span })
        .then_ignore(end())
}

fn binary_expr(lhs: Expr, (op, rhs): (ast::BinaryOp, Expr)) -> Expr {
    let span = lhs.span.start..rhs.span.end;
    Expr::new(ExprKind::Binary { op, left: Box::new(lhs), right: Box::new(rhs) }, span)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex;
    use chumsky::stream::Stream;
    use std::path::Path;

    #[test]
    fn test_parser() {
        let source = "fn main() { mut x := 5 \n x = 7 \n println(x) }";
        let tokens = lex::tokenize(source, Path::new("<test>")).unwrap();
        let eoi = source.len()..source.len();
        let program = parser().parse(Stream::from_iter(eoi, tokens.into_iter())).unwrap();
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "main");
        assert_eq!(program.functions[0].body.len(), 3);

        assert!(matches!(
            &program.functions[0].body[0].kind,
            StmtKind::VarDecl { name, is_mut: true, expr, .. }
                if name == "x" && matches!(expr.kind, ExprKind::IntLiteral(5))
        ));

        assert!(matches!(
            &program.functions[0].body[1].kind,
            StmtKind::Assign { name, expr, .. }
                if name == "x" && matches!(expr.kind, ExprKind::IntLiteral(7))
        ));

        match &program.functions[0].body[2].kind {
            StmtKind::ExprStmt(Expr { kind: ExprKind::FunctionCall { name, args }, .. }) => {
                assert_eq!(name, "println");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0].kind, ExprKind::Identifier(ref name) if name == "x"));
            }
            _ => panic!("Expected ExprStmt(FunctionCall)"),
        }
    }

    #[test]
    fn parser_retains_ast_byte_spans() {
        let source = "fn main() { value := 5 }";
        let tokens = lex::tokenize(source, Path::new("<test>")).unwrap();
        let program = parser()
            .parse(Stream::from_iter(source.len()..source.len(), tokens.into_iter()))
            .unwrap();

        let StmtKind::VarDecl { name_span, expr, .. } = &program.functions[0].body[0].kind else {
            panic!("expected var declaration");
        };

        assert_eq!(name_span.clone(), 12..17);
        assert_eq!(expr.span, 21..22);
    }

    #[test]
    fn parser_accepts_initial_module_declaration() {
        let source = "module main\n\nfn main() {}";
        let tokens = lex::tokenize(source, Path::new("<test>")).unwrap();
        let program = parser()
            .parse(Stream::from_iter(source.len()..source.len(), tokens.into_iter()))
            .unwrap();

        let module = program.module.expect("expected module declaration");
        assert_eq!(module.name, "main");
        assert_eq!(module.name_span, 7..11);
        assert_eq!(module.span, 0..11);
    }

    #[test]
    fn parser_accepts_simple_import_declaration_from_module_import_docs() {
        // V docs: https://docs.vlang.io/module-imports.html#module-imports,
        // simple imports use `import module_name`.
        let source = "import os\n\nfn main() {}";
        let tokens = lex::tokenize(source, Path::new("<test>")).unwrap();
        let parsed =
            parser().parse(Stream::from_iter(source.len()..source.len(), tokens.into_iter()));

        assert!(parsed.is_ok(), "simple import declarations should parse: {parsed:?}");
    }

    #[test]
    fn parser_rejects_non_initial_module_declaration() {
        let source = "fn main() {}\nmodule other";
        let tokens = lex::tokenize(source, Path::new("<test>")).unwrap();
        let errors = parser()
            .parse(Stream::from_iter(source.len()..source.len(), tokens.into_iter()))
            .unwrap_err();

        assert_eq!(errors[0].span(), 13..19);
    }
}
