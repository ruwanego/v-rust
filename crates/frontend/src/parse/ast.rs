use crate::source::Span;

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub module: Option<ModuleDecl>,
    pub imports: Vec<ImportDecl>,
    pub functions: Vec<FunctionDecl>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleDecl {
    pub name: String,
    pub name_span: Span,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportDecl {
    pub name: String,
    pub name_span: Span,
    pub symbols: Vec<ImportSymbol>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportSymbol {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub name_span: Span,
    pub return_type: Option<TypeName>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeName {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl Stmt {
    #[must_use]
    pub fn new(kind: StmtKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    ExprStmt(Expr),
    VarDecl { name: String, name_span: Span, is_mut: bool, expr: Expr },
    Assign { name: String, name_span: Span, expr: Expr },
    Return { expr: Expr },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Minus,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    #[must_use]
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    StringLiteral(String),
    IntLiteral(i64),
    BoolLiteral(bool),
    Identifier(String),
    FunctionCall { name: String, args: Vec<Expr> },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Unary { op: UnaryOp, expr: Box<Expr> },
}
