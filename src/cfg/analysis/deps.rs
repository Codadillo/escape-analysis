use std::{collections::{HashMap, HashSet}, fmt::Debug, hash::Hash};

use crate::{
    ast::Ident,
    cfg::{Cfg, Statement, Value},
};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct DepGraph<T> {
    pub place: usize,
    pub weight: Option<T>,
    pub deps: Option<Deps<T>>,
    pub dep_ty: DepType,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Deps<T> {
    All(Vec<DepGraph<T>>),
    Xor(Vec<DepGraph<T>>),
    Function(Ident, Vec<DepGraph<T>>),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum DepType {
    // aliases its children (i.e. we can pretend it is its children)
    Transparent,

    // aliases its children, but the childrens' places are from a different function and thus should be ignored
    TransparentLocked,
    // uses its children
    Depend,
}

impl<T> DepGraph<T> {
    pub fn leaf(place: usize) -> Self {
        Self {
            place,
            weight: None,
            deps: None,
            dep_ty: DepType::Depend,
        }
    }

    pub fn dummy_deps(deps: Deps<T>) -> Self {
        Self::weightless(0, deps)
    }

    pub fn weightless(place: usize, deps: Deps<T>) -> Self {
        Self {
            place,
            weight: None,
            deps: Some(deps),
            dep_ty: DepType::Depend,
        }
    }

    pub fn transparent(&self) -> bool {
        match self.dep_ty {
            DepType::Transparent | DepType::TransparentLocked => true,
            DepType::Depend => false,
        }
    }

    pub fn with_dep_ty(mut self, dep_ty: DepType) -> Self {
        self.dep_ty = dep_ty;
        self
    }

    pub fn rename(self, map: &HashMap<usize, usize>) -> Self {
        Self {
            place: *map.get(&self.place).unwrap_or(&self.place),
            weight: self.weight,
            deps: self.deps.map(|deps| match deps {
                Deps::All(ds) => Deps::All(ds.into_iter().map(|d| d.rename(map)).collect()),
                Deps::Xor(ds) => Deps::Xor(ds.into_iter().map(|d| d.rename(map)).collect()),
                Deps::Function(name, ds) => {
                    Deps::Function(name, ds.into_iter().map(|d| d.rename(map)).collect())
                }
            }),
            dep_ty: self.dep_ty,
        }
    }
}

impl<T: Debug + Eq + Hash> DepGraph<T> {
    pub fn squash(&mut self, old_lives: &HashSet<usize>) {
        if self.dep_ty == DepType::Transparent {
            self.dep_ty = DepType::TransparentLocked;
        }

        match &mut self.deps {
            Some(Deps::Xor(deps) | Deps::All(deps) | Deps::Function(_, deps)) => {
                for dep in &mut *deps {
                    dep.squash(old_lives);
                }
            }
            None => {}
        }

        if let Some(Deps::Xor(deps)) = &mut self.deps {
            let mut classes = HashMap::new();
            let extra_deps: Vec<_> = deps
                .iter()
                .enumerate()
                .filter_map(|(i, dep)| {
                    classes.insert(
                        match dep.dep_ty {
                            DepType::Depend if old_lives.contains(&dep.place) => Ok(dep.place),
                            _ => Err((&dep.weight, &dep.deps)),
                        },
                        i,
                    )
                })
                .collect();

            for &i in extra_deps.iter().rev() {
                println!("{:?}", deps[i]);
                deps.remove(i);
            }
        };

        match &mut self.deps {
            Some(Deps::Xor(deps) | Deps::All(deps))
                if deps.len() == 1 && self.dep_ty != DepType::Depend =>
            {
                *self = deps.pop().unwrap();
            }
            _ => {}
        }
    }
}

impl<T: Clone + Debug> DepGraph<T> {
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
                dep_ty: DepType::Transparent,
            };
        }

        for stmnt in cfg.statements() {
            let dep = match stmnt {
                Statement::Assign(a) => match &a.value {
                    Value::Place(d) => {
                        DepGraph::weightless(a.place, Deps::All(vec![DepGraph::leaf(*d)]))
                            .with_dep_ty(DepType::Transparent)
                    }
                    Value::Call { func, args } => DepGraph::weightless(
                        a.place,
                        Deps::Function(
                            func.clone(),
                            args.iter().copied().map(DepGraph::leaf).collect(),
                        ),
                    ),
                },
                Statement::Nop => continue,
            };

            let place = dep.place;
            base_deps[place] = dep;
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
            Some(Deps::Function(name, ..)) => panic!(
                "Cannot flatten unsubstituted call node: _{} > {name:?}(..)",
                self.place
            ),
            None => {}
        }

        if self.dep_ty != DepType::TransparentLocked {
            out[self.place] += 1;
        }
    
        out
    }

    pub fn meld(&mut self, reference: &Vec<DepGraph<T>>) {
        if let Some(Deps::All(deps) | Deps::Xor(deps)) = &mut self.deps {
            for dep in deps {
                if dep.dep_ty != DepType::TransparentLocked {
                    *dep = reference[dep.place].clone();
                }

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

impl<T: Debug> Debug for DepGraph<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DG {{ ")?;

        match self.dep_ty {
            DepType::Transparent => write!(f, "T({})", self.place)?,
            DepType::TransparentLocked => write!(f, "L({})", self.place)?,
            DepType::Depend => write!(f, "{}", self.place)?,
        }

        write!(f, ", {:?}", self.weight)?;

        if let Some(deps) = &self.deps {
            write!(f, ", ")?;

            let (name, deps) = match deps {
                Deps::All(deps) => ("D::All".to_owned(), deps),
                Deps::Xor(deps) => ("D::Xor".to_owned(), deps),
                Deps::Function(name, deps) => (name.0.clone(), deps),
            };

            write!(f, "{name}(")?;
            for (i, dep) in deps.iter().enumerate() {
                write!(f, "{dep:?}")?;

                if i + 1 != deps.len() {
                    write!(f, ", ")?;
                }
            }
            write!(f, ")")?;
        }

        write!(f, " }}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_squash() {
        let mut g = DepGraph {
            place: 0,
            weight: None::<()>,
            deps: Some(Deps::Xor(vec![
                DepGraph::weightless(99, Deps::All(vec![DepGraph::leaf(3)])),
                DepGraph::weightless(100, Deps::All(vec![DepGraph::leaf(3)])),
            ])),
            dep_ty: DepType::Transparent,
        };

        g.squash(&HashSet::new());

        assert_eq!(
            g,
            DepGraph {
                place: 0,
                weight: None,
                deps: Some(Deps::All(vec![DepGraph::leaf(3)])),
                dep_ty: DepType::Depend,
            },
        )
    }
}
