pub mod deps;

use std::collections::HashMap;

use self::deps::DepGraph;

use super::Cfg;
use crate::ast::Ident;

pub struct Context {
    pub fns: HashMap<Ident, Function>,
}

pub struct Function {
    pub cfg: Cfg,
    pub deps: Option<DepGraph>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            fns: HashMap::new(),
        }
    }

    pub fn add_cfgs(&mut self, cfgs: impl IntoIterator<Item = Cfg>) {
        self.fns.extend(
            cfgs.into_iter()
                .map(|cfg| (cfg.name.clone(), Function { cfg, deps: None })),
        )
    }

    pub fn get_cfg(&self, ident: &Ident) -> Option<&Cfg> {
        self.fns.get(ident).map(|f| &f.cfg)
    }

    pub fn compute_depgraph(&mut self, ident: &Ident) -> Option<DepGraph> {
        let func = self.fns.get(ident)?;
        if let Some(deps) = &func.deps {
            return Some(deps.clone());
        }

        let cfg = func.cfg.clone();
        let mut deps = DepGraph::from_cfg(self, &cfg);
        deps.simplify(&(1..=cfg.arg_count).collect::<Vec<_>>());

        self.fns.get_mut(ident)?.deps = Some(deps.clone());

        Some(deps)
    }
}
