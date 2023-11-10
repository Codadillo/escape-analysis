use std::collections::HashMap;

use crate::ast::Ident;

use super::{
    deps::{DepGraph, Deps},
    lra::Perm,
    signature::{ArgLives, ReturnLives},
};

#[derive(Debug)]
pub struct PolySig {
    any: Option<(usize, ReturnLives)>,
    mono: HashMap<ArgLives, ReturnLives>,
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
        let sig = self.function_sigs.get(name)?;
        sig.mono.get(args).or_else(|| {
            sig.any
                .as_ref()
                .filter(|(count, _)| *count == args.arg_count())
                .map(|(_, r)| r)
        })
    }

    pub fn insert_constructor(&mut self, name: Ident, arg_count: usize) {
        self.function_sigs.insert(
            name,
            PolySig {
                any: Some((
                    arg_count,
                    ReturnLives::new(
                        DepGraph {
                            place: arg_count + 1,
                            weight: Some(Perm::Shared),
                            deps: Some(Deps::All((1..=arg_count).map(DepGraph::leaf).collect())),
                        },
                        &(1..=arg_count).collect(),
                    ),
                )),
                mono: HashMap::from_iter([(
                    ArgLives::new((1..=arg_count).map(|a| (a, Perm::Exclusive)).collect()),
                    ReturnLives::new(
                        DepGraph {
                            place: arg_count + 1,
                            weight: Some(Perm::Exclusive),
                            deps: Some(Deps::All((1..=arg_count).map(DepGraph::leaf).collect())),
                        },
                        &(1..=arg_count).collect(),
                    ),
                )]),
            },
        );
    }
}
