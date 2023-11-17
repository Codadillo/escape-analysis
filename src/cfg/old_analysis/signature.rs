use std::collections::{BTreeMap, HashMap, HashSet};

use crate::cfg::old_analysis::deps::Deps;

use super::{deps::DepGraph, lra::Perm};

#[derive(Clone, Debug)]
pub struct Signature {
    pub args: ArgLives,
    pub ret: ReturnLives,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ArgLives {
    pub perms: BTreeMap<usize, Perm>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReturnLives {
    pub graph: DepGraph<Perm>,
    pub new_lives: HashSet<usize>,
    pub perms: HashMap<usize, Perm>,
}

impl Signature {
    pub fn new(arg_perms: &[Perm], ret_graph: DepGraph<Perm>) -> Self {
        let args = ArgLives::from_direct(arg_perms);
        let ret = ReturnLives::new(ret_graph, &args.perms.keys().copied().collect());

        Self { ret, args }
    }
}

impl ArgLives {
    pub fn new(perms: BTreeMap<usize, Perm>) -> Self {
        Self { perms }
    }

    pub fn from_direct(arg_perms: &[Perm]) -> Self {
        Self {
            perms: arg_perms
                .iter()
                .copied()
                .enumerate()
                .map(|(i, p)| (i + 1, p))
                .collect(),
        }
    }

    pub fn arg_count(&self) -> usize {
        self.perms.len()
    }
}

impl ReturnLives {
    pub fn new(graph: DepGraph<Perm>, old_lives: &HashSet<usize>) -> Self {
        fn traverse(
            graph: &DepGraph<Perm>,
            old_lives: &HashSet<usize>,
            new_lives: &mut HashSet<usize>,
            perms: &mut HashMap<usize, Perm>,
        ) {
            if !graph.transparent() {
                if !old_lives.contains(&graph.place) {
                    new_lives.insert(graph.place);
                }
    
                perms.insert(graph.place, graph.weight.unwrap());
            }

            match &graph.deps {
                Some(Deps::All(deps) | Deps::Xor(deps)) => {
                    for dep in deps {
                        traverse(dep, old_lives, new_lives, perms);
                    }
                }
                Some(Deps::Function(name, _)) => {
                    panic!("Unexpected function dependenyc {name} in returnlives")
                }
                None => {}
            }
        }

        let mut new_lives = HashSet::new();
        let mut perms = HashMap::new();

        traverse(&graph, old_lives, &mut new_lives, &mut perms);

        Self {
            graph,
            new_lives,
            perms,
        }
    }
}
