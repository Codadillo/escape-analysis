use std::fs;

use perm_mem::{
    cfg::{
        analysis::lra::{Perm, LRA},
        Cfg,
    },
    parser,
};

fn main() {
    let input = fs::read_to_string("testsimple.rs").unwrap();

    let parser = parser::FunctionParser::new();
    let ast = match parser.parse(&input) {
        Ok(a) => a,
        Err(e) => panic!("{e}"),
    };

    let cfg = Cfg::from_ast(ast);
    let lra = LRA::analyze(
        &cfg,
        (1..=cfg.arg_count).map(|p| (p, Perm::Exclusive)).collect(),
    );

    for (i, g) in lra.dep_graphs.iter().enumerate() {
        println!("{i}: {:?}", g.flatten(&cfg));
    }

    let mut plva: Vec<_> = lra.lva.point_lva(&cfg).into_iter().collect();
    plva.sort_by_key(|(i, _)| *i);
    for (p, lv) in plva {
        println!("{p:?} <@ {lv:?}");
    }

    let mut plra: Vec<_> = lra.plra.iter().collect();
    plra.sort_by_key(|(i, _)| *i);
    for (p, lr) in plra {
        println!("{p:?} <@ {lr:?}");
    }

    println!("------------CFG----------------");
    println!("{cfg:?}");
}
