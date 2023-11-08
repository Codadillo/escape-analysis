use crate::cfg;

use super::{BasicBlock, Call, Cfg, Phi, PlaceValue, Statement, Terminator, Value};

impl Cfg {
    pub fn from_base(cfg: &cfg::Cfg) -> Self {
        Self {
            arg_count: cfg.arg_count,
            place_count: cfg.place_count,
            basic_blocks: cfg.basic_blocks.iter().map(BasicBlock::from_base).collect(),
        }
    }
}

impl BasicBlock {
    fn from_base(bb: &cfg::BasicBlock) -> Self {
        Self {
            phi: bb.phi.iter().map(Phi::from_base).collect(),
            stmnts: bb.stmnts.iter().map(Statement::from_base).collect(),
            terminator: bb.terminator.as_ref().map(Terminator::from_base),
        }
    }
}

impl Phi {
    fn from_base(phi: &cfg::Phi) -> Self {
        Self {
            place: phi.place,
            opts: phi
                .opts
                .iter()
                .map(|(s, o)| (*s, PlaceValue::Ref(*o)))
                .collect(),
        }
    }
}

impl Statement {
    fn from_base(stmnt: &cfg::Statement) -> Self {
        match stmnt {
            cfg::Statement::Assign(assign) => Self {
                place: assign.place,
                value: Value::from_base(&assign.value),
            },
        }
    }
}

impl Value {
    fn from_base(value: &cfg::Value) -> Self {
        match value {
            cfg::Value::Place(p) => Self::Place(PlaceValue::Ref(*p)),
            cfg::Value::Call { func, args } => Self::Call(Call {
                func: func.clone(),
                args: args.iter().map(|&a| PlaceValue::Ref(a)).collect(),
            }),
        }
    }
}

impl Terminator {
    fn from_base(term: &cfg::Terminator) -> Self {
        match term {
            cfg::Terminator::Goto(b) => Self::Goto(*b),
            cfg::Terminator::Return => Self::Return,
            cfg::Terminator::IfElse { cond, iff, elsee } => Self::IfElse {
                cond: PlaceValue::Ref(*cond),
                iff: *iff,
                elsee: *elsee,
            },
        }
    }
}
