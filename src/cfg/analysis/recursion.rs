use crate::cfg::{Assign, BasicBlock, Cfg, Statement, Value};

use super::{deps::DepGraph, Context};

impl Context {
    pub fn compute_recursive_depgraph(&mut self, cfg: &Cfg) -> DepGraph {
        let mut no_recurse = cfg.clone();
        let mut recurses = false;

        for i in 0..no_recurse.basic_blocks.len() {
            if no_recurse.basic_blocks[i].stmnts.iter().any(|s| matches!(
                s,
                Statement::Assign(Assign { value: Value::Call { func, .. }, .. }) if func == &cfg.name
            )) {
                kill_linear_path(&mut no_recurse, i);
                recurses = true;
            }
        }

        let args = (0..=cfg.arg_count).collect::<Vec<_>>();

        let mut deps = DepGraph::from_cfg(self, &no_recurse);
        deps.simplify(&args);
        if !recurses {
            return deps;
        }

        for _ in 0..50 {
            self.set_depgraph(&cfg.name, deps);

            let mut next = DepGraph::from_cfg(self, &cfg);
            next.simplify(&args);

            if &next == self.get_depgraph(&cfg.name).unwrap() {
                return next;
            }

            deps = next;
        }

        dot::render(
            self.get_depgraph(&cfg.name).unwrap(),
            &mut std::fs::File::create(&format!("renders/0convergence_failure.{}.dot", cfg.name))
                .unwrap(),
        )
        .unwrap();

        dot::render(
            &deps,
            &mut std::fs::File::create(&format!("renders/1convergence_failure.{}.dot", cfg.name))
                .unwrap(),
        )
        .unwrap();

        panic!("Could not converge");
    }
}

fn kill_linear_path(cfg: &mut Cfg, start: usize) {
    let mut stack = vec![start];
    let preds_map = cfg.predecessors();

    while let Some(i) = stack.pop() {
        let succs = cfg.successors(i);
        let bb = &mut cfg.basic_blocks[i];

        if i == start || preds_map[&i].len() == 1 {
            *bb = BasicBlock {
                phi: vec![],
                stmnts: vec![],
                terminator: None,
            };

            for succ in succs {
                stack.push(succ);

                for phi in &mut cfg.basic_blocks[succ].phi {
                    phi.opts.remove(&i);
                }
            }
        }
    }
}