use std::collections::HashMap;

use crate::ast;

pub mod from_cfg;

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

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum PlaceValue {
    Move(usize),
    Ref(usize),
}

#[derive(Clone)]
pub struct Phi {
    pub place: usize,
    pub opts: HashMap<usize, PlaceValue>,
}

#[derive(Clone)]
pub struct Statement {
    pub place: usize,
    pub value: Value,
}

#[derive(Clone)]
pub enum Value {
    Place(PlaceValue),
    Call(Call),
}

#[derive(Clone)]
pub struct Call {
    pub func: ast::Ident,
    pub args: Vec<PlaceValue>,
}

#[derive(Clone)]
pub enum Terminator {
    Goto(usize),
    Return,
    IfElse {
        cond: PlaceValue,
        iff: usize,
        elsee: usize,
    },
}
