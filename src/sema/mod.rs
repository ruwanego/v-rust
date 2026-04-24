use crate::parse::ast::{Expr, Program, Stmt};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SemaError {
    pub message: String,
}

pub struct SemanticAnalyzer {
    pub variables: HashMap<String, VarInfo>,
}

pub struct VarInfo {
    pub is_mut: bool,
    pub typ: String,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            variables: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), Vec<SemaError>> {
        let mut errors = Vec::new();

        for func in &program.functions {
            // Function scope: clear for each function (no globals yet)
            self.variables.clear();

            for stmt in &func.body {
                if let Err(e) = self.analyze_stmt(stmt) {
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) -> Result<(), SemaError> {
        match stmt {
            Stmt::ExprStmt(expr) => {
                self.analyze_expr(expr)?;
                Ok(())
            }
            Stmt::VarDecl { name, is_mut, expr } => {
                if self.variables.contains_key(name) {
                    return Err(SemaError {
                        message: format!("Variable '{}' is already declared (shadowing is not allowed in V).", name),
                    });
                }
                let typ = self.analyze_expr(expr)?;
                self.variables.insert(name.clone(), VarInfo {
                    is_mut: *is_mut,
                    typ,
                });
                Ok(())
            }
            Stmt::Assign { name, expr } => {
                if let Some(var_info) = self.variables.get(name) {
                    if !var_info.is_mut {
                        return Err(SemaError {
                            message: format!("Variable '{}' is immutable. Declare it with `mut {} := ...` to assign to it.", name, name),
                        });
                    }
                    let typ = self.analyze_expr(expr)?;
                    if typ != var_info.typ {
                        return Err(SemaError {
                            message: format!("Type mismatch: cannot assign type '{}' to variable '{}' of type '{}'.", typ, name, var_info.typ),
                        });
                    }
                } else {
                    return Err(SemaError {
                        message: format!("Variable '{}' is not declared.", name),
                    });
                }
                Ok(())
            }
        }
    }

    fn analyze_expr(&self, expr: &Expr) -> Result<String, SemaError> {
        match expr {
            Expr::IntLiteral(_) => Ok("i64".to_string()),
            Expr::StringLiteral(_) => Ok("String".to_string()),
            Expr::BoolLiteral(_) => Ok("bool".to_string()),
            Expr::Identifier(name) => {
                if let Some(var_info) = self.variables.get(name) {
                    Ok(var_info.typ.clone())
                } else {
                    Err(SemaError {
                        message: format!("Variable '{}' is not declared.", name),
                    })
                }
            }
            Expr::FunctionCall { name, args } => {
                if name == "println" {
                    for arg in args {
                        self.analyze_expr(arg)?;
                    }
                    Ok("void".to_string())
                } else {
                    Err(SemaError {
                        message: format!("Unknown function '{}'.", name),
                    })
                }
            }
            Expr::Binary { op, left, right } => {
                let left_type = self.analyze_expr(left)?;
                let right_type = self.analyze_expr(right)?;

                use crate::parse::ast::BinaryOp;

                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        if left_type != "i64" || right_type != "i64" {
                            return Err(SemaError {
                                message: format!("Arithmetic operator {:?} requires both sides to be integers. Found {} and {}", op, left_type, right_type),
                            });
                        }
                        Ok("i64".to_string())
                    }
                    BinaryOp::Eq | BinaryOp::NotEq => {
                        if left_type != right_type {
                            return Err(SemaError {
                                message: format!("Relational operator {:?} requires both sides to be of the same type. Found {} and {}", op, left_type, right_type),
                            });
                        }
                        Ok("bool".to_string())
                    }
                    BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => {
                        if left_type != "i64" || right_type != "i64" {
                            return Err(SemaError {
                                message: format!("Relational operator {:?} requires both sides to be integers. Found {} and {}", op, left_type, right_type),
                            });
                        }
                        Ok("bool".to_string())
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if left_type != "bool" || right_type != "bool" {
                            return Err(SemaError {
                                message: format!("Logical operator {:?} requires both sides to be booleans. Found {} and {}", op, left_type, right_type),
                            });
                        }
                        Ok("bool".to_string())
                    }
                }
            }
            Expr::Unary { op, expr } => {
                let expr_type = self.analyze_expr(expr)?;
                use crate::parse::ast::UnaryOp;
                match op {
                    UnaryOp::Minus => {
                        if expr_type != "i64" {
                            return Err(SemaError {
                                message: format!("Unary minus requires an integer. Found {}", expr_type),
                            });
                        }
                        Ok("i64".to_string())
                    }
                    UnaryOp::Not => {
                        if expr_type != "bool" {
                            return Err(SemaError {
                                message: format!("Logical NOT requires a boolean. Found {}", expr_type),
                            });
                        }
                        Ok("bool".to_string())
                    }
                }
            }
        }
    }
}
