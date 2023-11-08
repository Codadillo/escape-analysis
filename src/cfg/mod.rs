pub mod analysis;
pub mod from_ast;
pub mod render;

use std::collections::HashMap;

use crate::ast;
use from_ast::ConversionState;

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
    Dead(usize),
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

impl Cfg {
    pub fn from_ast(func: ast::Function) -> Self {
        ConversionState::from_ast(func)
    }

    pub fn with_args(arg_count: usize) -> Self {
        Self {
            arg_count: arg_count,
            place_count: arg_count + 1,
            basic_blocks: vec![BasicBlock {
                phi: vec![],
                stmnts: vec![],
                terminator: None,
            }],
        }
    }

    pub fn add_place(&mut self) -> usize {
        let out = self.place_count;
        self.place_count += 1;
        out
    }

    pub fn add_bb(&mut self) -> usize {
        let out = self.basic_blocks.len();
        self.basic_blocks.push(BasicBlock {
            phi: vec![],
            stmnts: vec![],
            terminator: None,
        });
        out
    }

    pub fn statements(&self) -> impl Iterator<Item = &Statement> {
        self.basic_blocks.iter().map(|b| &b.stmnts).flatten()
    }

    pub fn get_statement(&self, p: (usize, usize)) -> Option<&Statement> {
        self.basic_blocks.get(p.0).and_then(|b| b.stmnts.get(p.1))
    }

    pub fn statements_idx(&self) -> impl Iterator<Item = ((usize, usize), &Statement)> {
        self.basic_blocks
            .iter()
            .enumerate()
            .map(|(i, b)| b.stmnts.iter().enumerate().map(move |(j, s)| ((i, j), s)))
            .flatten()
    }

    pub fn successors(&self, block: usize) -> Vec<usize> {
        match &self.basic_blocks[block].terminator {
            Some(Terminator::Goto(b)) => vec![*b],
            Some(Terminator::IfElse { iff, elsee, .. }) => vec![*iff, *elsee],
            _ => vec![],
        }
    }

    pub fn predecessors(&self) -> HashMap<usize, Vec<usize>> {
        let mut pred_map: HashMap<_, Vec<_>> = HashMap::new();

        for (i, block) in self.basic_blocks.iter().enumerate() {
            match &block.terminator {
                Some(Terminator::Goto(next)) => pred_map.entry(*next).or_default().push(i),
                Some(Terminator::IfElse {
                    cond: _,
                    iff,
                    elsee,
                }) => {
                    pred_map.entry(*iff).or_default().push(i);
                    pred_map.entry(*elsee).or_default().push(i);
                }
                _ => {}
            }
        }

        pred_map
    }
}
