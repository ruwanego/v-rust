use crate::parse::ast::{BinaryOp, Expr, ExprKind, Program, Stmt, StmtKind, UnaryOp};
use crate::source::Span;
use crate::types::Type;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct CheckedProgram {
    pub functions: Vec<CheckedFunction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckedFunction {
    pub name: String,
    pub body: Vec<CheckedStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CheckedStmt {
    ExprStmt { expr: CheckedExpr, span: Span },
    VarDecl { name: String, is_mut: bool, typ: Type, expr: CheckedExpr, span: Span },
    Assign { name: String, typ: Type, expr: CheckedExpr, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckedExpr {
    pub typ: Type,
    pub span: Span,
    pub kind: CheckedExprKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CheckedExprKind {
    StringLiteral(String),
    IntLiteral(i64),
    BoolLiteral(bool),
    Identifier(String),
    FunctionCall { name: String, args: Vec<CheckedExpr> },
    Binary { op: BinaryOp, left: Box<CheckedExpr>, right: Box<CheckedExpr> },
    Unary { op: UnaryOp, expr: Box<CheckedExpr> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemaError {
    pub kind: SemaErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemaErrorKind {
    AlreadyDeclared { name: String },
    ImmutableAssignment { name: String },
    UndeclaredVariable { name: String },
    TypeMismatch { name: String, expected: Type, actual: Type },
    UnknownFunction { name: String },
    InvalidArithmeticOperands { op: BinaryOp, left: Type, right: Type },
    InvalidEqualityOperands { op: BinaryOp, left: Type, right: Type },
    InvalidRelationalOperands { op: BinaryOp, left: Type, right: Type },
    InvalidLogicalOperands { op: BinaryOp, left: Type, right: Type },
    InvalidUnaryOperand { op: UnaryOp, actual: Type },
}

impl fmt::Display for SemaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            SemaErrorKind::AlreadyDeclared { name } => {
                write!(
                    formatter,
                    "Variable '{name}' is already declared (shadowing is not allowed in V)."
                )
            }
            SemaErrorKind::ImmutableAssignment { name } => {
                write!(
                    formatter,
                    "Variable '{name}' is immutable. Declare it with `mut {name} := ...` to assign to it."
                )
            }
            SemaErrorKind::UndeclaredVariable { name } => {
                write!(formatter, "Variable '{name}' is not declared.")
            }
            SemaErrorKind::TypeMismatch { name, expected, actual } => {
                write!(
                    formatter,
                    "Type mismatch: cannot assign type '{actual}' to variable '{name}' of type '{expected}'."
                )
            }
            SemaErrorKind::UnknownFunction { name } => {
                write!(formatter, "Unknown function '{name}'.")
            }
            SemaErrorKind::InvalidArithmeticOperands { op, left, right } => {
                write!(
                    formatter,
                    "Arithmetic operator {op:?} requires both sides to be integers. Found {left} and {right}"
                )
            }
            SemaErrorKind::InvalidEqualityOperands { op, left, right } => {
                write!(
                    formatter,
                    "Relational operator {op:?} requires both sides to be of the same type. Found {left} and {right}"
                )
            }
            SemaErrorKind::InvalidRelationalOperands { op, left, right } => {
                write!(
                    formatter,
                    "Relational operator {op:?} requires both sides to be integers. Found {left} and {right}"
                )
            }
            SemaErrorKind::InvalidLogicalOperands { op, left, right } => {
                write!(
                    formatter,
                    "Logical operator {op:?} requires both sides to be booleans. Found {left} and {right}"
                )
            }
            SemaErrorKind::InvalidUnaryOperand { op, actual } => match op {
                UnaryOp::Minus => {
                    write!(formatter, "Unary minus requires an integer. Found {actual}")
                }
                UnaryOp::Not => write!(formatter, "Logical NOT requires a boolean. Found {actual}"),
            },
        }
    }
}

pub struct SemanticAnalyzer {
    pub variables: HashMap<String, VarInfo>,
}

pub struct VarInfo {
    pub is_mut: bool,
    pub typ: Type,
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzer {
    #[must_use]
    pub fn new() -> Self {
        SemanticAnalyzer { variables: HashMap::new() }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<CheckedProgram, Vec<SemaError>> {
        let mut errors = Vec::new();
        let mut functions = Vec::new();

        for func in &program.functions {
            self.variables.clear();
            let mut body = Vec::new();

            for stmt in &func.body {
                match self.analyze_stmt(stmt) {
                    Ok(checked_stmt) => body.push(checked_stmt),
                    Err(error) => errors.push(error),
                }
            }

            functions.push(CheckedFunction { name: func.name.clone(), body });
        }

        if errors.is_empty() {
            Ok(CheckedProgram { functions })
        } else {
            Err(errors)
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) -> Result<CheckedStmt, SemaError> {
        match stmt {
            Stmt { kind: StmtKind::ExprStmt(expr), span } => {
                Ok(CheckedStmt::ExprStmt { expr: self.analyze_expr(expr)?, span: span.clone() })
            }
            Stmt { kind: StmtKind::VarDecl { name, name_span, is_mut, expr }, span } => {
                if self.variables.contains_key(name) {
                    return Err(error(
                        SemaErrorKind::AlreadyDeclared { name: name.clone() },
                        name_span.clone(),
                    ));
                }

                let checked_expr = self.analyze_expr(expr)?;
                let typ = checked_expr.typ;
                self.variables.insert(name.clone(), VarInfo { is_mut: *is_mut, typ });

                Ok(CheckedStmt::VarDecl {
                    name: name.clone(),
                    is_mut: *is_mut,
                    typ,
                    expr: checked_expr,
                    span: span.clone(),
                })
            }
            Stmt { kind: StmtKind::Assign { name, name_span, expr }, span } => {
                let var_info = self.variables.get(name).ok_or_else(|| {
                    error(
                        SemaErrorKind::UndeclaredVariable { name: name.clone() },
                        name_span.clone(),
                    )
                })?;

                if !var_info.is_mut {
                    return Err(error(
                        SemaErrorKind::ImmutableAssignment { name: name.clone() },
                        name_span.clone(),
                    ));
                }

                let checked_expr = self.analyze_expr(expr)?;
                if checked_expr.typ != var_info.typ {
                    return Err(error(
                        SemaErrorKind::TypeMismatch {
                            name: name.clone(),
                            expected: var_info.typ,
                            actual: checked_expr.typ,
                        },
                        expr.span.clone(),
                    ));
                }

                Ok(CheckedStmt::Assign {
                    name: name.clone(),
                    typ: var_info.typ,
                    expr: checked_expr,
                    span: span.clone(),
                })
            }
        }
    }

    fn analyze_expr(&self, expr: &Expr) -> Result<CheckedExpr, SemaError> {
        match &expr.kind {
            ExprKind::IntLiteral(value) => {
                Ok(checked_expr(Type::I64, expr.span.clone(), CheckedExprKind::IntLiteral(*value)))
            }
            ExprKind::StringLiteral(value) => Ok(checked_expr(
                Type::String,
                expr.span.clone(),
                CheckedExprKind::StringLiteral(value.clone()),
            )),
            ExprKind::BoolLiteral(value) => Ok(checked_expr(
                Type::Bool,
                expr.span.clone(),
                CheckedExprKind::BoolLiteral(*value),
            )),
            ExprKind::Identifier(name) => {
                let var_info = self.variables.get(name).ok_or_else(|| {
                    error(
                        SemaErrorKind::UndeclaredVariable { name: name.clone() },
                        expr.span.clone(),
                    )
                })?;
                Ok(checked_expr(
                    var_info.typ,
                    expr.span.clone(),
                    CheckedExprKind::Identifier(name.clone()),
                ))
            }
            ExprKind::FunctionCall { name, args } => {
                if name != "println" {
                    return Err(error(
                        SemaErrorKind::UnknownFunction { name: name.clone() },
                        expr.span.clone(),
                    ));
                }

                let checked_args =
                    args.iter().map(|arg| self.analyze_expr(arg)).collect::<Result<Vec<_>, _>>()?;

                Ok(checked_expr(
                    Type::Void,
                    expr.span.clone(),
                    CheckedExprKind::FunctionCall { name: name.clone(), args: checked_args },
                ))
            }
            ExprKind::Binary { op, left, right } => {
                let checked_left = self.analyze_expr(left)?;
                let checked_right = self.analyze_expr(right)?;
                let typ =
                    check_binary_op(*op, checked_left.typ, checked_right.typ, expr.span.clone())?;

                Ok(checked_expr(
                    typ,
                    expr.span.clone(),
                    CheckedExprKind::Binary {
                        op: *op,
                        left: Box::new(checked_left),
                        right: Box::new(checked_right),
                    },
                ))
            }
            ExprKind::Unary { op, expr: inner } => {
                let checked_inner = self.analyze_expr(inner)?;
                let typ = check_unary_op(*op, checked_inner.typ, expr.span.clone())?;

                Ok(checked_expr(
                    typ,
                    expr.span.clone(),
                    CheckedExprKind::Unary { op: *op, expr: Box::new(checked_inner) },
                ))
            }
        }
    }
}

fn checked_expr(typ: Type, span: Span, kind: CheckedExprKind) -> CheckedExpr {
    CheckedExpr { typ, span, kind }
}

fn check_binary_op(op: BinaryOp, left: Type, right: Type, span: Span) -> Result<Type, SemaError> {
    match op {
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
            if left.is_integer() && right.is_integer() {
                Ok(Type::I64)
            } else {
                Err(error(SemaErrorKind::InvalidArithmeticOperands { op, left, right }, span))
            }
        }
        BinaryOp::Eq | BinaryOp::NotEq => {
            if left == right {
                Ok(Type::Bool)
            } else {
                Err(error(SemaErrorKind::InvalidEqualityOperands { op, left, right }, span))
            }
        }
        BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => {
            if left.is_integer() && right.is_integer() {
                Ok(Type::Bool)
            } else {
                Err(error(SemaErrorKind::InvalidRelationalOperands { op, left, right }, span))
            }
        }
        BinaryOp::And | BinaryOp::Or => {
            if left.is_bool() && right.is_bool() {
                Ok(Type::Bool)
            } else {
                Err(error(SemaErrorKind::InvalidLogicalOperands { op, left, right }, span))
            }
        }
    }
}

fn check_unary_op(op: UnaryOp, actual: Type, span: Span) -> Result<Type, SemaError> {
    match op {
        UnaryOp::Minus => {
            if actual.is_integer() {
                Ok(Type::I64)
            } else {
                Err(error(SemaErrorKind::InvalidUnaryOperand { op, actual }, span))
            }
        }
        UnaryOp::Not => {
            if actual.is_bool() {
                Ok(Type::Bool)
            } else {
                Err(error(SemaErrorKind::InvalidUnaryOperand { op, actual }, span))
            }
        }
    }
}

fn error(kind: SemaErrorKind, span: Span) -> SemaError {
    SemaError { kind, span }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lex, parse};
    use chumsky::stream::Stream;
    use chumsky::Parser;
    use std::path::Path;

    #[test]
    fn analyzer_returns_checked_program_with_expression_types() {
        let program =
            parse_program("fn main() { mut value := 5 value = value + 2 println(value) }");
        let mut analyzer = SemanticAnalyzer::new();

        let checked = analyzer.analyze(&program).unwrap();

        assert_eq!(checked.functions.len(), 1);
        assert_eq!(checked.functions[0].body.len(), 3);
        assert!(matches!(
            checked.functions[0].body[0],
            CheckedStmt::VarDecl { typ: Type::I64, .. }
        ));
        assert!(matches!(checked.functions[0].body[1], CheckedStmt::Assign { typ: Type::I64, .. }));
        assert!(matches!(
            checked.functions[0].body[2],
            CheckedStmt::ExprStmt { expr: CheckedExpr { typ: Type::Void, .. }, .. }
        ));
    }

    #[test]
    fn analyzer_reports_structured_type_mismatch() {
        let program = parse_program("fn main() { mut value := 5 value = 'nope' }");
        let mut analyzer = SemanticAnalyzer::new();

        let errors = analyzer.analyze(&program).unwrap_err();

        assert_eq!(
            errors[0].kind,
            SemaErrorKind::TypeMismatch {
                name: "value".to_string(),
                expected: Type::I64,
                actual: Type::String,
            }
        );
        assert_eq!(errors[0].span, 35..41);
    }

    #[test]
    fn analyzer_reports_undeclared_assignment_span() {
        let program = parse_program("fn main() { value = 7 }");
        let mut analyzer = SemanticAnalyzer::new();

        let errors = analyzer.analyze(&program).unwrap_err();

        assert_eq!(errors[0].kind, SemaErrorKind::UndeclaredVariable { name: "value".to_string() });
        assert_eq!(errors[0].span, 12..17);
    }

    fn parse_program(source: &str) -> Program {
        let tokens = lex::tokenize(source, Path::new("<test>")).unwrap();
        parse::parser()
            .parse(Stream::from_iter(source.len()..source.len(), tokens.into_iter()))
            .unwrap()
    }
}
