use std::collections::{BTreeMap, HashMap, HashSet};

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

#[derive(Clone, Debug)]
pub struct ReturnLives {
    pub graph: DepGraph<Perm>,
    pub new_lives: HashSet<usize>,
    pub perms: HashMap<usize, Perm>,
}

impl Signature {
    pub fn new(arg_perms: &[Perm], ret_graph: DepGraph<Perm>) -> Self {
        let args: BTreeMap<_, _> = arg_perms
            .iter()
            .copied()
            .enumerate()
            .map(|(i, p)| (i + 1, p))
            .collect();

        Self {
            ret: ReturnLives::new(ret_graph, &args.keys().copied().collect()),
            args: ArgLives::new(args),
        }
    }
}

impl ArgLives {
    pub fn new(perms: BTreeMap<usize, Perm>) -> Self {
        Self { perms }
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
            if !old_lives.contains(&graph.place) {
                new_lives.insert(graph.place);
            }

            perms.insert(graph.place, graph.weight.unwrap());
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

    pub fn empty_newlives(graph: DepGraph<Perm>) -> Self {
        let mut this = Self::new(graph, &HashSet::new());
        this.new_lives = HashSet::new();
        this
    }
}
