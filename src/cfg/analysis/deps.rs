use crate::cfg::{Cfg, Statement, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepGraph<T> {
    pub place: usize,
    pub weight: Option<T>,
    pub deps: Option<Deps<T>>,
}

impl<T> DepGraph<T> {
    fn leaf(place: usize) -> Self {
        Self {
            place,
            weight: None,
            deps: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Deps<T> {
    All(Vec<DepGraph<T>>),
    Xor(Vec<DepGraph<T>>),
}

impl<T: Clone> DepGraph<T> {
    pub fn analyze(cfg: &Cfg) -> Vec<Self> {
        let base_deps = Self::base_deps(cfg);

        base_deps
            .iter()
            .enumerate()
            .map(|(place, deps)| {
                let mut graph = Self {
                    place,
                    weight: None,
                    deps: deps.clone(),
                };
                graph.meld(&base_deps);
                graph
            })
            .collect()
    }

    pub fn base_deps(cfg: &Cfg) -> Vec<Option<Deps<T>>> {
        let mut base_deps: Vec<_> = vec![None; cfg.place_count];

        for phi in cfg.basic_blocks.iter().flat_map(|b| &b.phi) {
            base_deps[phi.place] = Some(Deps::Xor(
                phi.opts.values().map(|&v| DepGraph::leaf(v)).collect(),
            ));
        }

        for stmnt in cfg.statements() {
            let (p, deps) = match stmnt {
                Statement::Assign(a) => (
                    a.place,
                    match &a.value {
                        Value::Place(d) => Deps::All(vec![DepGraph::leaf(*d)]),
                        Value::Call { args, .. } => {
                            Deps::All(args.iter().copied().map(DepGraph::leaf).collect())
                        }
                    },
                ),
            };

            base_deps[p] = Some(deps);
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
            None => {}
        }

        out[self.place] += 1;
        out
    }

    pub fn meld(&mut self, reference: &Vec<Option<Deps<T>>>) {
        if let Some(Deps::All(deps) | Deps::Xor(deps)) = &mut self.deps {
            for dep in deps {
                dep.deps = reference[dep.place].clone();
                dep.meld(reference);
            }
        }
    }
}

pub fn add_ctrs(a: &mut Vec<usize>, b: &Vec<usize>) {
    for (i, j) in a.iter_mut().zip(b) {
        *i += j;
    }
}
