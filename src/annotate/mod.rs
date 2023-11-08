use std::collections::HashMap;

use crate::ast;

#[derive(Clone)]
pub struct Cfg {
    pub arg_count: usize,
    pub place_count: usize,
    pub basic_blocks: Vec<BasicBlock>,
}

#[derive(Clone)]
pub struct BasicBlock {
    pub phi: Vec<Phi>,
    pub stmnts: Vec<Statement>,
    pub terminator: Option<Terminator>,
}

#[derive(Clone)]
pub enum Statement {
    Assign(Assign),
}

#[derive(Clone)]
pub struct Phi {
    pub place: usize,
    pub opts: HashMap<usize, usize>,
}

#[derive(Clone)]
pub struct Assign {
    pub place: usize,
    pub value: Value,
}

#[derive(Clone)]
pub enum Value {
    Place(usize),
    Call { func: ast::Ident, args: Vec<usize> },
}

#[derive(Clone)]
pub enum Terminator {
    Goto(usize),
    Return,
    IfElse {
        cond: usize,
        iff: usize,
        elsee: usize,
    },
}
