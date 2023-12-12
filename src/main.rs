use std::{collections::HashMap, env, fs, path::PathBuf};

use perm_mem::{
    backend::compile_module_to_dir,
    cfg::{
        analysis::{deps::DepGraph, Context},
        mem_manage, Cfg,
    },
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

    let mut type_map = module.ty_defs;
    type_map.extend(
        module
            .fns
            .iter()
            .map(|f| (f.name.0.clone(), f.ret_ty.clone())),
    );

    let mut ctx = Context::new();
    ctx.add_cfgs(
        module
            .fns
            .into_iter()
            .map(|f| Cfg::from_ast(f, type_map.clone())),
    );
    ctx.type_map = type_map;

    let names: Vec<_> = ctx.fns.iter().map(|(n, _)| n.clone()).collect();
    let mut managed_cfgs = HashMap::new();
    for name in names {
        let mut cfg = ctx.get_cfg(&name).unwrap().clone();
        // println!("{name}: {:?}\n", cfg);

        // println!("{:?}", DepGraph::from_cfg(&mut ctx, &cfg, true));
        let ret_alloced = ctx.compute_depgraph(&name).unwrap().nodes[0].allocated();
        let deps = DepGraph::from_cfg(&mut ctx, &cfg, ret_alloced);

        {
            let deps = deps.clone();
            // deps.simplify(&[]);

            fs::create_dir_all("renders/").unwrap();
            dot::render(
                &deps,
                &mut std::fs::File::create(&format!(
                    "renders/{}.{name}.dot",
                    path.file_name().unwrap().to_str().unwrap()
                ))
                .unwrap(),
            )
            .unwrap();
        }

        mem_manage::insert_management(&mut ctx, &mut cfg);
        println!("{name}: {:?}\n", cfg);
        managed_cfgs.insert(cfg.name.clone(), (cfg, deps));
    }

    compile_module_to_dir("build", managed_cfgs, &ctx.type_map).unwrap();
}
