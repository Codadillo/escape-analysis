use std::{env, fs, path::PathBuf};

use perm_mem::{
    cfg::{analysis::{Context, deps::DepGraph}, Cfg, mem_manage},
    parser,
};

fn main() {
    let path = PathBuf::from(env::args().nth(1).unwrap());
    let input = fs::read_to_string(&path).unwrap();

    let parser = parser::ModuleParser::new();
    let module = match parser.parse(&input) {
        Ok(m) => m,
        Err(e) => panic!("{e}"),
    };

    let mut ctx = Context::new();
    ctx.add_cfgs(module.into_iter().map(Cfg::from_ast));

    let names: Vec<_> = ctx.fns.iter().map(|(n, _)| n.clone()).collect();
    for name in names {
        let mut cfg = ctx.get_cfg(&name).unwrap().clone();
        // println!("{name}: {:?}\n", cfg);

        println!("{:?}", DepGraph::from_cfg(&mut ctx, &cfg, true));

        mem_manage::insert_management(&mut ctx, &mut cfg);
        println!("{name}: {:?}\n", cfg);

        let graph = ctx.compute_depgraph(&name).unwrap();
        println!("{graph:?}");
    
        // dot::render(
        //     &graph,
        //     &mut std::fs::File::create(&format!(
        //         "renders/{}.{name}.dot",
        //         path.file_name().unwrap().to_str().unwrap()
        //     ))
        //     .unwrap(),
        // )
        // .unwrap();
    }
}
