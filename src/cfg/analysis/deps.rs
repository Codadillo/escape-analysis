use std::collections::HashMap;

use crate::{
    ast::Ident,
    cfg::{Cfg, Statement, Value},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepGraph<T> {
    pub place: usize,
    pub weight: Option<T>,
    pub deps: Option<Deps<T>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Deps<T> {
    All(Vec<DepGraph<T>>),
    Xor(Vec<DepGraph<T>>),
    Func(Ident, Vec<DepGraph<T>>),
}

impl<T> DepGraph<T> {
    pub fn leaf(place: usize) -> Self {
        Self {
            place,
            weight: None,
            deps: None,
        }
    }

    pub fn rename(self, map: &HashMap<usize, usize>) -> Self {
        Self {
            place: map[&self.place],
            weight: self.weight,
            deps: self.deps.map(|deps| match deps {
                Deps::All(ds) => Deps::All(ds.into_iter().map(|d| d.rename(map)).collect()),
                Deps::Xor(ds) => Deps::Xor(ds.into_iter().map(|d| d.rename(map)).collect()),
                Deps::Func(name, ds) => {
                    Deps::Func(name, ds.into_iter().map(|d| d.rename(map)).collect())
                }
            }),
        }
    }
}

impl<T: Clone> DepGraph<T> {
    pub fn analyze(cfg: &Cfg) -> Vec<Self> {
        let mut base_deps = Self::base_deps(cfg);
        let reference = base_deps.clone();

        for graph in &mut base_deps {
            graph.meld(&reference);
        }

        base_deps
    }

    pub fn base_deps(cfg: &Cfg) -> Vec<DepGraph<T>> {
        let mut base_deps: Vec<_> = (0..cfg.place_count).map(DepGraph::leaf).collect();

        for phi in cfg.basic_blocks.iter().flat_map(|b| &b.phi) {
            base_deps[phi.place] = DepGraph {
                place: phi.place,
                weight: None,
                deps: Some(Deps::Xor(
                    phi.opts.values().map(|&v| DepGraph::leaf(v)).collect(),
                )),
            };
        }

        for stmnt in cfg.statements() {
            let (p, deps) = match stmnt {
                Statement::Assign(a) => (
                    a.place,
                    match &a.value {
                        Value::Place(d) => Deps::All(vec![DepGraph::leaf(*d)]),
                        Value::Call { func, args } => {
                            Deps::Func(
                                func.clone(),
                                args.iter().copied().map(DepGraph::leaf).collect(),
                            )
                            // Deps::All(args.iter().copied().map(DepGraph::leaf).collect())
                        }
                    },
                ),
                Statement::Nop => continue,
            };

            base_deps[p] = DepGraph {
                place: p,
                weight: None,
                deps: Some(deps),
            };
        }

        base_deps
    }

    pub fn flatten(&self, cfg: &Cfg) -> Vec<usize> {
        self.flatten_to_ctrs(cfg)
            .iter()
            .enumerate()
            .flat_map(|(i, d)| std::iter::repeat(i).take(*d))
            .collect()
    }

    pub fn flatten_to_ctrs(&self, cfg: &Cfg) -> Vec<usize> {
        let mut out = vec![0; cfg.place_count];

        match &self.deps {
            Some(Deps::All(deps)) => {
                for dep in deps {
                    add_ctrs(&mut out, &dep.flatten_to_ctrs(cfg));
                }
            }
            Some(Deps::Xor(deps)) => {
                for dep in deps {
                    for (fdep, out_dep) in dep.flatten_to_ctrs(cfg).into_iter().zip(&mut out) {
                        *out_dep = fdep.max(*out_dep);
                    }
                }
            }
            Some(Deps::Func(name, ..)) => panic!(
                "Cannot flatten unsubstituted call node: _{} > {name:?}(..)",
                self.place
            ),
            None => {}
        }

        out[self.place] += 1;
        out
    }

    pub fn meld(&mut self, reference: &Vec<DepGraph<T>>) {
        if let Some(Deps::All(deps) | Deps::Xor(deps)) = &mut self.deps {
            for dep in deps {
                *dep = reference[dep.place].clone();
                dep.meld(reference);
            }
        }
    }
}

pub fn add_ctrs(a: &mut [usize], b: &[usize]) {
    for (i, j) in a.iter_mut().zip(b) {
        *i += j;
    }
}
