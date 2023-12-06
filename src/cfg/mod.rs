pub mod analysis;
pub mod from_ast;
pub mod mem_manage;
pub mod render;

use std::collections::HashMap;

use crate::{ast, types::Type};
use from_ast::ConversionState;

#[derive(Clone)]
pub struct Cfg {
    pub name: ast::Ident,
    pub arg_count: usize,
    pub place_tys: Vec<Type>,
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
    Deallocate(usize),
    Dup(RefCount),
    Drop(RefCount),
    Nop,
}

#[derive(Clone)]
pub struct RefCount {
    pub place: usize,
    pub count: usize,
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
    pub allocate: bool,
}

#[derive(Clone)]
pub enum Value {
    Place(usize),
    Call { func: ast::Ident, args: Vec<usize> },
}

#[derive(Clone)]
pub enum Terminator {
    Goto(usize),
    Return(usize),
    IfElse {
        cond: usize,
        iff: usize,
        elsee: usize,
    },
}

impl Cfg {
    pub fn from_ast(func: ast::Function, type_map: HashMap<String, Type>) -> Self {
        ConversionState::from_ast(func, type_map)
    }

    pub fn with_args(name: ast::Ident, args: Vec<Type>, ret_ty: Type) -> Self {
        let arg_count = args.len();
        let mut place_tys = args;
        place_tys.insert(0, ret_ty);

        Self {
            name,
            arg_count,
            place_tys,
            basic_blocks: vec![BasicBlock {
                phi: vec![],
                stmnts: vec![],
                terminator: None,
            }],
        }
    }

    pub fn add_place(&mut self, ty: Type) -> usize {
        let out = self.place_tys.len();
        self.place_tys.push(ty);

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

    /// This could not cover every block if unreachable blocks exist.
    pub fn bb_order(&self) -> Vec<usize> {
        let mut order = vec![0];
        let mut focus = 0;
        while let Some(block) = order.get(focus) {
            order.extend(self.successors(*block));
            focus += 1;
        }

        order
    }

    pub fn statements(&self) -> impl Iterator<Item = &Statement> {
        self.basic_blocks.iter().flat_map(|b| &b.stmnts)
    }

    pub fn get_statement(&self, p: (usize, usize)) -> Option<&Statement> {
        self.basic_blocks.get(p.0).and_then(|b| b.stmnts.get(p.1))
    }

    pub fn statements_idx(&self) -> impl Iterator<Item = ((usize, usize), &Statement)> {
        self.basic_blocks
            .iter()
            .enumerate()
            .flat_map(|(i, b)| b.stmnts.iter().enumerate().map(move |(j, s)| ((i, j), s)))
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

impl RefCount {
    pub fn one(place: usize) -> Self {
        Self { place, count: 1 }
    }
}

impl BasicBlock {
    // this iterator can produce the same place multiple times
    pub fn phi_used_vars(&self) -> impl Iterator<Item = &usize> {
        self.phi.iter().flat_map(|phi| phi.opts.values())
    }
}
