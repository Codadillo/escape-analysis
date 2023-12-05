use std::collections::HashSet;

use super::{
    analysis::{
        deps::DepGraph,
        lva::LVA,
        Context,
    },
    Cfg, RefCount, Statement, Terminator, Value,
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
        let mut drop_locations = vec![];

        // If there are no preds and any arguments aren't live, drop them
        if preds.get(&b).filter(|p| !p.is_empty()).is_none() {
            for &dead_arg in args.difference(&lva.blocks[b].live_in) {
                if deps.nodes[dead_arg].allocated() {
                    drop_locations.push((0, RefCount::one(dead_arg)));
                }
            }
        }

        // Go through the block statement by statement
        let mut live_out = lva.blocks[b].live_out.clone();
        for (i, stmnt) in cfg.basic_blocks[b].stmnts.iter_mut().enumerate().rev() {
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
                            if place_alloced
                                && ctx
                                    .get_depgraph(func)
                                    .map(|d| !d.nodes[0].allocated())
                                    .unwrap_or(true)
                            {
                                a.allocate = true;
                            }

                            args.iter().copied().collect()
                        }
                    }
                }
                Statement::Nop => continue,
                Statement::Deallocate(_) | Statement::Dup(_) | Statement::Drop(_) => todo!(),
            };

            let live_ctrs = deps.flatten_to_counters(live_out.iter().copied());

            for &new in &new_live_in {
                if live_ctrs[new] == 0 && deps.nodes[new].allocated() {
                    drop_locations.push((i + 1, RefCount::one(new)));
                }
            }

            live_out.extend(new_live_in);
        }

        // Actually add in the operations
        for (loc, count) in drop_locations.into_iter().rev() {
            cfg.basic_blocks[b]
                .stmnts
                .insert(loc, Statement::Drop(count));
        }

        // If the terminator of this uses a variable that's
        // not in live out, drop it at the start of each successor
        match cfg.basic_blocks[b].terminator {
            Some(Terminator::IfElse { cond, .. }) if !lva.blocks[b].live_out.contains(&cond) => {
                for succ in cfg.successors(b) {
                    cfg.basic_blocks[succ]
                        .stmnts
                        .insert(0, Statement::Drop(RefCount::one(cond)))
                }
            }
            _ => {}
        }
    }
}
