use crate::cfg::{
    analysis::{lra::LRA, signature::Signature},
    Assign, BasicBlock, Cfg, Statement, Value,
};

use super::{context::Context, signature::ArgLives};

pub fn compute_recursive_lra(ctx: &mut Context, cfg: &Cfg, monomorph: &ArgLives) -> LRA {
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

    let mut lra = LRA::analyze(ctx, &no_recurse, monomorph);
    if !recurses {
        return lra;
    }

    loop {
        ctx.set_mono_sig(
            cfg.name.clone(),
            Signature {
                args: monomorph.clone(),
                ret: lra.ret.clone(),
            },
        );

        let next = LRA::analyze(ctx, cfg, monomorph);

        if next == lra {
            return lra;
        }

        lra = next;
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
