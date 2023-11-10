use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(pub String);

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Ident,
    pub args: Vec<Ident>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmnts: Vec<Statement>,
    pub ret: Expr,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub ident: Ident,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Ident(Ident),
    Call(Call),
    Block(Box<Block>),
    IfElse(Box<IfElse>),
}

#[derive(Debug, Clone)]
pub struct IfElse {
    pub cond: Expr,
    pub iff: Block,
    pub elsee: Block,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub ident: Ident,
    pub args: Vec<Expr>,
}

impl<S: Into<String>> From<S> for Ident {
    fn from(value: S) -> Self {
        Ident(value.into())
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
