use std::collections::HashMap;

use crate::{ast::Ident, cfg::Cfg};

use super::{
    deps::{DepGraph, DepType, Deps},
    lra::Perm,
    recursion,
    signature::{ArgLives, ReturnLives, Signature},
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

    pub fn compute_sig(&mut self, name: &Ident, args: &ArgLives) -> Option<ReturnLives> {
        if let Some(ret) = self.get_sig(name, args) {
            return Some(ret);
        }

        let lra = recursion::compute_recursive_lra(self, &self.cfgs.get(name)?.clone(), &args);

        self.function_sigs
            .entry(name.clone())
            .or_default()
            .mono
            .insert(args.clone(), lra.ret.clone());
        Some(lra.ret)
    }

    pub fn set_mono_sig(&mut self, name: Ident, sig: Signature) {
        self.function_sigs
            .entry(name)
            .or_default()
            .mono
            .insert(sig.args, sig.ret);
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
                        dep_ty: DepType::Depend,
                    })
                    .collect();

                Some(ReturnLives::new(
                    DepGraph {
                        place: 0,
                        weight: Some(Perm::Exclusive),
                        deps: Some(Deps::All(deps)),
                        dep_ty: DepType::Depend,
                    },
                    &(1..=args.arg_count()).collect(),
                ))
            }
            "invent" => Some(ReturnLives::new(
                DepGraph {
                    place: 0,
                    weight: Some(Perm::Exclusive),
                    deps: None,
                    dep_ty: DepType::Depend,
                },
                &(1..=args.arg_count()).collect(),
            )),
            _ => None,
        }
    }
}
