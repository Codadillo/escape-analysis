use std::collections::HashMap;

use crate::ast::Ident;

use super::{
    deps::{DepGraph, Deps},
    lra::Perm,
    signature::{ArgLives, ReturnLives},
};

#[derive(Debug)]
pub enum PolySig {
    Any(usize, ReturnLives),
    Mono(HashMap<ArgLives, ReturnLives>),
}

#[derive(Debug)]
pub struct Context {
    pub function_sigs: HashMap<Ident, PolySig>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            function_sigs: HashMap::new(),
        }
    }

    pub fn get_sig(&self, name: &Ident, args: &ArgLives) -> Option<&ReturnLives> {
        match self.function_sigs.get(name)? {
            PolySig::Any(count, ret) => (*count == args.arg_count()).then_some(ret),
            PolySig::Mono(mono) => mono.get(args),
        }
    }

    pub fn insert_any(&mut self, name: Ident, arg_count: usize, perm: Perm) {
        self.function_sigs.insert(
            name,
            PolySig::Any(
                arg_count,
                ReturnLives::new(
                    DepGraph {
                        place: arg_count,
                        weight: Some(perm),
                        deps: Some(Deps::All((1..arg_count).map(DepGraph::leaf).collect())),
                    },
                    &(1..arg_count).collect(),
                ),
            ),
        );
    }
}
