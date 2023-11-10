use std::{collections::HashMap, fs};

use perm_mem::{
    cfg::{
        analysis::{
            context::Context,
            lra::{Perm, LRA},
        },
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

    let mut ctx = Context {
        function_sigs: HashMap::new(),
    };
    ctx.insert_constructor("function1".into(), 1);
    ctx.insert_constructor("function2".into(), 2);
    ctx.insert_constructor("function3".into(), 3);
    ctx.insert_constructor("function4".into(), 4);

    let cfg = Cfg::from_ast(ast);
    let lra = LRA::analyze(
        &mut ctx,
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
