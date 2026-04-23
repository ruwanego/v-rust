#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub functions: Vec<FunctionDecl>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDecl {
    pub name: String,
}
