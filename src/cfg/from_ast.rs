use std::collections::HashMap;

use super::{Assign, Phi, Statement, Terminator, Value};
use crate::{ast, cfg::Cfg};

pub struct ConversionState {
    pub cfg: Cfg,
    pub scopes: Vec<HashMap<ast::Ident, usize>>,
    pub last_block: usize,
}

impl ConversionState {
    pub fn from_ast(func: ast::Function) -> Cfg {
        let mut this = ConversionState {
            cfg: Cfg::with_args(func.args.len()),
            scopes: Vec::new(),
            last_block: 0,
        };

        this.push_scope();
        for (i, arg) in func.args.into_iter().enumerate() {
            this.set_place_scoped(arg, i + 1);
        }

        let ret = this.add_block(func.body);
        this.add_assign(0, Value::Place(ret));
        this.set_terminator(Terminator::Return);

        this.cfg
    }

    pub fn add_expr(&mut self, expr: ast::Expr) -> usize {
        match expr {
            ast::Expr::Ident(id) => self
                .get_place_scoped(&id)
                .unwrap_or_else(|| panic!("Identifier {id:?} not found")),
            ast::Expr::Call(call) => self.add_call(call),
            ast::Expr::Block(b) => self.add_block(*b),
            ast::Expr::IfElse(ifelse) => self.add_ifelse(*ifelse),
        }
    }

    pub fn add_assign(&mut self, place: usize, value: Value) -> usize {
        self.cfg.basic_blocks[self.last_block]
            .stmnts
            .push(Statement::Assign(Assign { place, value }));
        place
    }

    pub fn set_terminator(&mut self, terminator: Terminator) {
        self.cfg.basic_blocks[self.last_block].terminator = Some(terminator);
    }

    pub fn add_call(&mut self, call: ast::Call) -> usize {
        let place = self.cfg.add_place();
        let args = call.args.into_iter().map(|e| self.add_expr(e)).collect();

        self.add_assign(
            place,
            Value::Call {
                func: call.ident,
                args,
            },
        )
    }

    pub fn add_ifelse(&mut self, ifelse: ast::IfElse) -> usize {
        let cond = self.add_expr(ifelse.cond);

        let if_block = self.cfg.add_bb();
        let else_block = self.cfg.add_bb();

        self.set_terminator(Terminator::IfElse {
            cond,
            iff: if_block,
            elsee: else_block,
        });

        let end_bb = self.cfg.add_bb();

        self.focus(if_block);
        let if_out = self.add_block(ifelse.iff);
        self.set_terminator(Terminator::Goto(end_bb));

        self.focus(else_block);
        let else_out = self.add_block(ifelse.elsee);
        self.set_terminator(Terminator::Goto(end_bb));

        self.focus(end_bb);
        self.add_phi(HashMap::from_iter([
            (if_block, if_out),
            (else_block, else_out),
        ]))
    }

    pub fn add_block(&mut self, block: ast::Block) -> usize {
        self.push_scope();

        for stmnt in block.stmnts {
            let value = self.add_expr(stmnt.value);
            self.set_place_scoped(stmnt.ident, value);
        }

        let ret = self.add_expr(block.ret);
        self.pop_scope();
        ret
    }

    pub fn add_phi(&mut self, opts: HashMap<usize, usize>) -> usize {
        let place = self.cfg.add_place();
        self.cfg.basic_blocks[self.last_block]
            .phi
            .push(Phi { place, opts });

        place
    }

    pub fn focus(&mut self, block: usize) {
        self.last_block = block;
    }

    pub fn get_place_scoped(&self, ident: &ast::Ident) -> Option<usize> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(ident))
            .copied()
    }

    pub fn set_place_scoped(&mut self, ident: ast::Ident, place: usize) {
        self.scopes.last_mut().unwrap().insert(ident, place);
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }
}
