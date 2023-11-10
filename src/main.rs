use std::fs;

use perm_mem::{
    cfg::{
        analysis::{context::Context, lra::Perm, recursion::UnRecurse, signature::ArgLives},
        Cfg,
    },
    parser,
};

fn main() {
    let input = fs::read_to_string("inputs/factorial.rs").unwrap();

    let parser = parser::ModuleParser::new();
    let module = match parser.parse(&input) {
        Ok(m) => m,
        Err(e) => panic!("{e}"),
    };

    let mut ctx = Context::new();
    ctx.cfgs.extend(
        module
            .into_iter()
            .map(|f| (f.name.clone(), Cfg::from_ast(f))),
    );

    // for (name, cfg) in ctx.cfgs.clone() {
    //     let args = ArgLives::from_direct(&vec![Perm::Exclusive; cfg.arg_count]);
    //     let ret = ctx.calculate_sig(&name, &args).unwrap();
    //     println!("fn {name}: {:?} <- {cfg:?}", ret.perms);
    // }

    let cfg = &ctx.cfgs[&"factorial".into()].clone();
    UnRecurse::analyze(
        &mut ctx,
        cfg,
        &ArgLives::from_direct(&vec![Perm::Exclusive; cfg.arg_count]),
    );

    // let lra = LRA::analyze(
    //     &mut ctx,
    //     &cfg,
    //     (1..=cfg.arg_count).map(|p| (p, Perm::Exclusive)).collect(),
    // );

    // for (i, g) in lra.dep_graphs.iter().enumerate() {
    //     println!("{i}: {:?}", g.flatten(&cfg));
    // }

    // let mut plva: Vec<_> = lra.lva.point_lva(&cfg).into_iter().collect();
    // plva.sort_by_key(|(i, _)| *i);
    // for (p, lv) in plva {
    //     println!("{p:?} <@ {lv:?}");
    // }

    // let mut plra: Vec<_> = lra.plra.iter().collect();
    // plra.sort_by_key(|(i, _)| *i);
    // for (p, lr) in plra {
    //     println!("{p:?} <@ {lr:?}");
    // }
}
