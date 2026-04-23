#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub functions: Vec<FunctionDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    ExprStmt(Expr),
    VarDecl {
        name: String,
        is_mut: bool,
        expr: Expr,
    },
    Assign {
        name: String,
        expr: Expr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    StringLiteral(String),
    IntLiteral(i64),
    Identifier(String),
    FunctionCall { name: String, args: Vec<Expr> },
}
