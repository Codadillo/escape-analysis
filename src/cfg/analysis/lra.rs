use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use crate::cfg::{
    analysis::deps::{add_ctrs, DepGraph},
    Cfg,
};

use super::{deps::Deps, lva::LVA, context::Context};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Perm {
    Exclusive,
    Shared,
    // Dynamic,
}

impl Debug for Perm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Perm::Exclusive => write!(f, "X"),
            Perm::Shared => write!(f, "S"),
            // Perm::Dynamic => write!(f, "D"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LRA {
    pub lva: LVA,
    pub plva: HashMap<(usize, isize), HashSet<usize>>,

    pub dep_graphs: Vec<DepGraph<Perm>>,
    pub plra: HashMap<(usize, isize), HashMap<usize, Perm>>,
}

impl LRA {
    pub fn analyze(ctx: &mut Context, cfg: &Cfg, monomorph: HashMap<usize, Perm>) -> Self {
        assert_eq!(monomorph.len(), cfg.arg_count);

        let lva = LVA::analyze(cfg);
        let plva = lva.point_lva(cfg);

        let mut graphs = DepGraph::<Perm>::analyze(ctx, cfg);
        let ctrs: Vec<Vec<_>> = graphs.iter().map(|g| g.flatten_to_ctrs(cfg)).collect();

        let mut plra = HashMap::new();

        for (p, live) in &plva {
            if p.1 < 0 {
                continue;
            }

            let live_ctrs =
                live.iter()
                    .map(|&l| &ctrs[l])
                    .fold(vec![0; cfg.place_count], |mut acc, ctr| {
                        add_ctrs(&mut acc, ctr);
                        acc
                    });

            let base_perms: Vec<_> = live_ctrs
                .into_iter()
                .enumerate()
                .map(|(i, ctr)| {
                    if ctr == 0 {
                        None
                    } else if monomorph.get(&i) != Some(&Perm::Shared) && ctr == 1 {
                        Some(Perm::Exclusive)
                    } else {
                        Some(Perm::Shared)
                    }
                })
                .collect();

            let mut live_refs = HashMap::new();
            for &lv in live {
                populate_perms(&mut graphs[lv], None, &base_perms, &mut live_refs);
            }

            plra.insert(*p, live_refs);
        }

        Self {
            lva,
            plva,
            plra,
            dep_graphs: graphs,
        }
    }
}

fn populate_perms(
    graph: &mut DepGraph<Perm>,
    parent: Option<Perm>,
    base_perms: &Vec<Option<Perm>>,
    out_perms: &mut HashMap<usize, Perm>,
) {
    use Perm::*;

    match (parent, base_perms[graph.place]) {
        (Some(Exclusive), Some(Exclusive) | None) => graph.weight = Some(Exclusive),
        (Some(_), _) => graph.weight = Some(Shared),
        (None, base_perm) => graph.weight = base_perm,
    }

    if let Some(perm) = out_perms.get(&graph.place) {
        assert_eq!(*perm, graph.weight.unwrap());
    } else {
        out_perms.insert(graph.place, graph.weight.unwrap());
    }

    if let Some(Deps::All(deps) | Deps::Xor(deps)) = &mut graph.deps {
        for dep in deps {
            populate_perms(dep, graph.weight, base_perms, out_perms);
        }
    }
}
