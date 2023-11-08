use std::collections::{HashMap, HashSet};

use crate::cfg::{
    Value, {BasicBlock, Cfg, Statement, Terminator},
};

use super::fixed_point::fixed_point;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct LVABlock {
    pub uevar: HashSet<usize>,
    pub def: HashSet<usize>,
    pub phi_out: HashSet<usize>,
    pub phi_in: HashSet<usize>,

    pub live_in: HashSet<usize>,
    pub live_out: HashSet<usize>,
}

impl LVABlock {
    fn new<'a>(i: usize, bb: &BasicBlock, succs: impl Iterator<Item = &'a BasicBlock>) -> Self {
        let mut this = Self::default();

        this.phi_out
            .extend(succs.flat_map(|b| &b.phi).map(|p| p.opts[&i]));

        this.phi_in
            .extend(bb.phi.iter().flat_map(|p| p.opts.values()));
        this.def.extend(bb.phi.iter().map(|p| p.place));

        for stmnt in &bb.stmnts {
            match &stmnt {
                Statement::Assign(assign) => {
                    match &assign.value {
                        Value::Place(p) => this.access(*p),
                        Value::Call { args, .. } => {
                            for &arg in args {
                                this.access(arg)
                            }
                        }
                    }

                    this.def.insert(assign.place);
                }
            };
        }

        if let Some(Terminator::IfElse { cond, .. }) = &bb.terminator {
            this.access(*cond);
        }

        this
    }

    fn access(&mut self, place: usize) {
        if !self.def.contains(&place) {
            self.uevar.insert(place);
        }
    }
}

#[derive(Debug, Clone)]
pub struct LVA {
    pub blocks: Vec<LVABlock>,
}

impl LVA {
    pub fn analyze(cfg: &Cfg) -> Self {
        let mut blocks: Vec<_> = cfg
            .basic_blocks
            .iter()
            .enumerate()
            .map(|(i, b)| {
                LVABlock::new(
                    i,
                    b,
                    cfg.successors(i).iter().map(|&i| &cfg.basic_blocks[i]),
                )
            })
            .collect();

        fixed_point(&mut blocks, |blocks, _old| {
            for i in 0..blocks.len() {
                let mut live_out = blocks[i].phi_out.clone();
                if let Some(Terminator::Return(p)) = &cfg.basic_blocks[i].terminator {
                    live_out.insert(*p);
                }
                live_out.extend(cfg.successors(i).iter().flat_map(|&b| &blocks[b].live_in));

                let block = &mut blocks[i];
                block.live_out = live_out;
                block.live_in = block
                    .uevar
                    .union(&block.live_out.difference(&block.def).cloned().collect())
                    .cloned()
                    .collect();
            }
        });

        Self { blocks }
    }

    pub fn point_lva(&self, cfg: &Cfg) -> HashMap<(usize, isize), HashSet<usize>> {
        let mut plva = HashMap::new();

        for (i, b) in cfg.basic_blocks.iter().enumerate() {
            let blva = &self.blocks[i];
            let mut living = blva.live_out.clone();

            for (j, stmnt) in b.stmnts.iter().enumerate().rev() {
                match stmnt {
                    Statement::Assign(a) => {
                        if living.remove(&a.place) {
                            match &a.value {
                                Value::Place(p) => living.extend([*p]),
                                Value::Call { args, .. } => living.extend(args),
                            };
                        }
                    }
                };

                plva.insert((i, j as isize), living.clone());
            }

            for phi in &b.phi {
                if living.remove(&phi.place) {
                    living.extend(phi.opts.values());
                }
            }
            plva.insert((i, -1), living);
            plva.insert((i, b.stmnts.len() as isize), blva.live_out.clone());
        }

        plva
    }
}
