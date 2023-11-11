use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Debug,
};

use crate::cfg::{
    analysis::{
        deps::{add_ctrs, DepGraph},
        signature::ArgLives,
    },
    Cfg, Terminator,
};

use super::{context::Context, deps::Deps, lva::LVA, signature::ReturnLives};

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

#[derive(Debug, Clone, PartialEq)]
pub struct LRA {
    pub lva: LVA,
    pub plva: HashMap<(usize, isize), HashSet<usize>>,

    pub graphs: Vec<DepGraph<Perm>>,
    pub plra: HashMap<(usize, isize), HashMap<usize, Perm>>,
    pub ret: ReturnLives,
}

impl LRA {
    pub fn analyze(ctx: &mut Context, cfg: &Cfg, monomorph: &ArgLives) -> Self {
        assert_eq!(monomorph.perms.len(), cfg.arg_count);

        let lva = LVA::analyze(cfg);
        let plva = lva.point_lva(cfg);

        let bb_order = cfg.bb_order();
        let mut sorted_plva: Vec<_> = plva.iter().collect();
        sorted_plva
            .sort_by_cached_key(|((b, s), _)| (bb_order.iter().find(|&a| a == b).unwrap(), s));

        let mut graphs = DepGraph::<Perm>::analyze(cfg);

        let mut plra = HashMap::new();

        let mut pred_perms = monomorph.perms.clone();
        for (p, live) in sorted_plva.iter().copied() {
            if p.1 < 0 {
                // TODO
                continue;
            }

            for &lv in live {            
                if let Some(Deps::Function(name, args)) = &graphs[lv].deps {
                    let mut arg_perms = BTreeMap::new();
                    let mut rename_map = HashMap::new();
                    for (i, arg) in args.iter().enumerate() {
                        arg_perms.insert(i + 1, pred_perms[&arg.place]);
                        rename_map.insert(i + 1, arg.place);
                    }

                    let sig = ctx.compute_sig(name, &ArgLives::new(arg_perms)).unwrap();

                    assert!(
                        sig.new_lives
                            .is_subset(&HashSet::from_iter([sig.graph.place])),
                        "{sig:?}"
                    );
                    if !(1..=args.len()).contains(&sig.graph.place) {
                        rename_map.insert(sig.graph.place, graphs[lv].place);
                    }

                    graphs[lv] = sig.graph.clone().rename(&rename_map);
                }
            }
    
            let reference = graphs.clone();
            for graph in &mut graphs {
                graph.meld(&reference);
            }

            let live_ctrs = live.iter().map(|&l| graphs[l].flatten_to_ctrs(cfg)).fold(
                vec![0; cfg.place_count],
                |mut acc, ctr| {
                    add_ctrs(&mut acc, &ctr);
                    acc
                },
            );

            let base_perms: Vec<_> = live_ctrs
                .into_iter()
                .enumerate()
                .map(|(i, ctr)| {
                    if ctr == 0 {
                        None
                    } else if monomorph.perms.get(&i) != Some(&Perm::Shared) && ctr == 1 {
                        Some(Perm::Exclusive)
                    } else {
                        Some(Perm::Shared)
                    }
                })
                .collect();

            let mut live_refs = HashMap::new();
            for &lv in live {
                populate_perms(&mut graphs[lv], None, &base_perms, &mut live_refs);
                pred_perms.insert(lv, graphs[lv].weight.unwrap());
            }

            plra.insert(*p, live_refs);
        }

        let ret_point = sorted_plva.last().unwrap().0;
        let ret_place = match cfg.basic_blocks[ret_point.0].terminator {
            Some(Terminator::Return(p)) => p,
            _ => panic!("Malformed cfg for {}: bad return block", cfg.name),
        };

        let mut graph = graphs[ret_place].clone();
        graph.squash();

        let ret = ReturnLives::new(graph, &(1..=cfg.arg_count).collect());

        Self {
            lva,
            plva,
            plra,
            ret,
            graphs,
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

    match &mut graph.deps {
        Some(Deps::All(deps) | Deps::Xor(deps)) => {
            for dep in deps {
                populate_perms(dep, graph.weight, base_perms, out_perms);
            }
        }
        Some(Deps::Function(name, _)) => panic!(
            "Unexpected Deps::Func while populating: {} > {name:?}(..)",
            graph.place
        ),
        None => {}
    }
}
