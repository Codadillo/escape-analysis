use std::collections::HashSet;

use super::{
    analysis::{deps::DepGraph, lva::LVA, Context},
    Cfg, RefCount, Statement, Value,
};

pub fn insert_management(ctx: &mut Context, cfg: &mut Cfg) {
    // compute the depgraph for the cfg
    let alloced_return = ctx.compute_depgraph(&cfg.name).unwrap().nodes[0].allocated();
    let deps = DepGraph::from_cfg(ctx, cfg, alloced_return);

    // compute the lva
    let lva = LVA::analyze(cfg);

    // insert dynamic management ops
    let args = HashSet::from_iter(1..=cfg.arg_count);
    let preds = cfg.predecessors();
    for b in 0..cfg.basic_blocks.len() {
        let mut added_stmnts = vec![];

        // If there are no preds and any arguments aren't live, drop them
        if preds.get(&b).filter(|p| !p.is_empty()).is_none() {
            for &dead_arg in args.difference(&lva.blocks[b].live_in) {
                if deps.nodes[dead_arg].allocated() {
                    added_stmnts.push((0, Statement::Drop(RefCount::one(dead_arg))));
                }
            }
        }

        // Go through the block statement by statement
        let mut live_out = lva.blocks[b].live_out.clone();
        for (i, stmnt) in cfg.basic_blocks[b].stmnts.iter_mut().enumerate().rev() {
            let live_ctrs = deps.flatten_to_counters_ignorant(live_out.iter().copied());
            let mut passed_ownership: HashSet<usize> = HashSet::new();

            let new_live_in: HashSet<_> = match stmnt {
                Statement::Assign(a) => {
                    let place_alloced = deps.nodes[a.place].allocated();

                    match &a.value {
                        Value::Place(p) => {
                            if place_alloced && !deps.nodes[*p].allocated() {
                                a.allocate = true;
                            }

                            [*p].into_iter().collect()
                        }
                        Value::Call { func, args } => {
                            let f_depgraph = ctx.compute_depgraph(func);
                            if place_alloced {
                                a.allocate = f_depgraph
                                    .as_ref()
                                    .map(|d| !d.nodes[0].allocated())
                                    .unwrap_or(true);
                            }

                            if let Some(f_depgraph) = f_depgraph {
                                let preorder: HashSet<_> =
                                    f_depgraph.preorder().into_iter().collect();

                                passed_ownership.extend(
                                    args.iter()
                                        .enumerate()
                                        .filter(|(i, _)| {
                                            let child_arg = i + 1;
                                            (a.allocate && preorder.contains(&child_arg))
                                                || f_depgraph.alloced_args.contains(&child_arg)
                                        })
                                        .map(|(_, arg)| arg),
                                );
                            } else if a.allocate && func.0 == "tuple" {
                                passed_ownership.extend(args);
                            }

                            args.iter().copied().collect()
                        }
                    }
                }
                Statement::Nop => continue,
                Statement::Deallocate(_) | Statement::Dup(_) | Statement::Drop(_) => continue,
            };

            let mut new_dups = vec![];
            let mut new_drops = vec![];
            for &new in &new_live_in {
                if !deps.nodes[new].allocated() {
                    continue;
                }

                let live_ref = live_ctrs[new] != 0 || live_out.contains(&new);

                if live_ref && passed_ownership.contains(&new) {
                    new_dups.push(new);
                }

                if !live_ref && !passed_ownership.contains(&new) {
                    new_drops.push(new);
                }
            }

            added_stmnts.extend(
                new_drops
                    .into_iter()
                    .map(|n| (i + 1, Statement::Drop(RefCount::one(n)))),
            );
            added_stmnts.extend(
                new_dups
                    .into_iter()
                    .map(|n| (i, Statement::Dup(RefCount::one(n)))),
            );

            live_out.extend(new_live_in);
        }

        // Actually add in the dup/drop operations
        for (loc, stmnt) in added_stmnts.into_iter() {
            cfg.basic_blocks[b].stmnts.insert(loc, stmnt);
        }

        // If any successors do not have a variable in their live ref in
        // that's in this block's live ref out, drop it on entry
        let live_ref_out = live_refs(&deps, &lva.blocks[b].live_out);
        for succ in cfg.successors(b) {
            let mut live_in = lva.blocks[succ].live_in.clone();
            live_in.extend(cfg.basic_blocks[succ].phi_used_vars());

            let live_ref_in = live_refs(&deps, &live_in);

            for dead in live_ref_out.difference(&live_ref_in) {
                cfg.basic_blocks[succ]
                    .stmnts
                    .insert(0, Statement::Drop(RefCount::one(*dead)));
            }
        }
    }
}

fn live_refs(deps: &DepGraph, lva: &HashSet<usize>) -> HashSet<usize> {
    let live_ctrs = deps.flatten_to_counters_ignorant(lva.iter().copied());
    (0..deps.nodes.len())
        .filter(|n| deps.nodes[*n].allocated())
        .filter(|n| live_ctrs[*n] != 0 || lva.contains(n))
        .collect()
}
