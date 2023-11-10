use std::collections::BTreeMap;

use crate::cfg::{Assign, BasicBlock, Cfg, Statement, Value, analysis::{lra::LRA, signature::Signature}};

use super::{lra::Perm, context::Context, signature::ArgLives};

pub struct UnRecurse {}

impl UnRecurse {
    pub fn analyze(ctx: &mut Context, cfg: &Cfg, monomorph: &ArgLives) -> Self {
        let mut no_recurse = cfg.clone();

        for i in 0..no_recurse.basic_blocks.len() {
            if no_recurse.basic_blocks[i].stmnts.iter().any(|s| matches!(
                s,
                Statement::Assign(Assign { value: Value::Call { func, .. }, .. }) if func == &cfg.name
            )) {
                kill_linear_path(&mut no_recurse, i);
            }
        }

        println!("{no_recurse:?}");

        let ground_truth = LRA::analyze(ctx, &no_recurse, monomorph);
        println!("g {:?}", ground_truth.ret);

        ctx.set_mono_sig(cfg.name.clone(), Signature {
            args: monomorph.clone(),
            ret: ground_truth.ret,
        });
        let next = LRA::analyze(ctx, cfg, monomorph);
        println!("n {:?}", next.ret);

        Self {}
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
