#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub functions: Vec<FunctionDecl>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub body: Vec<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    StringLiteral(String),
    IntLiteral(i64),
    FunctionCall { name: String, args: Vec<Expr> },
}
