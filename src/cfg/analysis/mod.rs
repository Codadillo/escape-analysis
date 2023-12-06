pub mod deps;
pub mod recursion;
pub mod lva;

use std::collections::HashMap;

use self::deps::DepGraph;

use super::Cfg;
use crate::{ast::Ident, types::Type};

pub struct Context {
    pub fns: HashMap<Ident, Function>,
    pub type_map: HashMap<String, Type>,
}

pub struct Function {
    pub cfg: Cfg,
    pub deps: Option<DepGraph>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            fns: HashMap::new(),
            type_map: HashMap::new(),
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

    pub fn set_depgraph(&mut self, ident: &Ident, deps: DepGraph) -> bool {
        match self.fns.get_mut(ident) {
            Some(f) => {
                f.deps = Some(deps);
                true
            }
            None => false,
        }
    }

    pub fn get_depgraph(&self, ident: &Ident) -> Option<&DepGraph> {
        self.fns.get(ident)?.deps.as_ref()
    }

    pub fn compute_depgraph(&mut self, ident: &Ident) -> Option<DepGraph> {
        let func = self.fns.get(ident)?;
        if let Some(deps) = &func.deps {
            return Some(deps.clone());
        }

        Some(self.compute_recursive_depgraph(&func.cfg.clone()))
    }
}
