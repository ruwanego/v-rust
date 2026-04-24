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
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Minus,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    StringLiteral(String),
    IntLiteral(i64),
    BoolLiteral(bool),
    Identifier(String),
    FunctionCall { name: String, args: Vec<Expr> },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
}
