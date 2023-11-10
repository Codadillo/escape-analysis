use std::collections::HashMap;

use crate::{
    ast::Ident,
    cfg::{Cfg, Terminator},
};

use super::{
    deps::{DepGraph, Deps},
    lra::{Perm, LRA},
    signature::{ArgLives, ReturnLives},
};

#[derive(Debug, Default)]
pub struct PolySig {
    any: Option<(usize, ReturnLives)>,
    mono: HashMap<ArgLives, ReturnLives>,
}

#[derive(Debug)]
pub struct Context {
    pub function_sigs: HashMap<Ident, PolySig>,
    pub cfgs: HashMap<Ident, Cfg>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            function_sigs: HashMap::new(),
            cfgs: HashMap::new(),
        }
    }

    pub fn calculate_sig(&mut self, name: &Ident, args: &ArgLives) -> Option<ReturnLives> {
        if let Some(ret) = self.get_sig(name, args) {
            return Some(ret);
        }

        let cfg = self.cfgs.get(name)?.clone();
        let mut lra = LRA::analyze(self, &cfg, &args.perms);

        let ret_block = *cfg.bb_order().last().unwrap();
        let ret_point = (ret_block, cfg.basic_blocks[ret_block].stmnts.len() as isize);
        let ret_place = match cfg.basic_blocks[ret_block].terminator {
            Some(Terminator::Return(p)) => p,
            _ => panic!("Malformed cfg for {name}: bad return block"),
        };

        let graph = lra.dep_graphs.remove(ret_place);
        let perms = lra.plra.remove(&ret_point).unwrap();
        let new_lives = perms
            .keys()
            .copied()
            .filter(|p| !(1..=cfg.arg_count).contains(p))
            .collect();

        let ret = ReturnLives {
            new_lives,
            graph,
            perms,
        };

        self.function_sigs
            .entry(name.clone())
            .or_default()
            .mono
            .insert(args.clone(), ret.clone());
        Some(ret)
    }

    pub fn get_sig(&self, name: &Ident, args: &ArgLives) -> Option<ReturnLives> {
        if let Some(sig) = self.intrinsic_sig(name, args) {
            return Some(sig);
        }

        let sig = self.function_sigs.get(name)?;
        sig.mono
            .get(args)
            .or_else(|| {
                sig.any
                    .as_ref()
                    .filter(|(count, _)| *count == args.arg_count())
                    .map(|(_, r)| r)
            })
            .cloned()
    }

    pub fn intrinsic_sig(&self, name: &Ident, args: &ArgLives) -> Option<ReturnLives> {
        match name.0.as_str() {
            "tuple" => {
                let deps = (1..=args.arg_count())
                    .map(|place| DepGraph {
                        place,
                        weight: Some(args.perms[&place]),
                        deps: None,
                    })
                    .collect();

                Some(ReturnLives::new(
                    DepGraph {
                        place: 0,
                        weight: Some(Perm::Exclusive),
                        deps: Some(Deps::All(deps)),
                    },
                    &(1..=args.arg_count()).collect(),
                ))
            }
            _ => None,
        }
    }
}
