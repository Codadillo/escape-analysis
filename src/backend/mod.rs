use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs,
    hash::{Hash, Hasher},
    io,
    io::Write,
    path::Path,
};

use crate::{
    cfg::{analysis::deps::DepGraph, Cfg, Statement, Terminator, Value},
    types::Type,
};

const MAKEFILE: &str = include_str!("Makefile");
const STD_BASE: &str = include_str!("std_base.c");

pub fn compile_module_to_dir<'a>(
    dir: impl AsRef<Path>,
    cfgs: impl IntoIterator<Item = &'a (Cfg, DepGraph)>,
    type_map: &HashMap<String, Type>,
) -> io::Result<()> {
    // Create the build directory
    let dir = dir.as_ref();
    fs::create_dir_all(dir)?;

    // Create our files
    let mut type_file = fs::File::create(dir.join("types.h"))?;
    writeln!(&mut type_file, "#pragma once")?;
    writeln!(&mut type_file, "#include <stdlib.h>\n")?;

    let std_file = fs::File::create(dir.join("std.c"))?;

    let mut program_file = fs::File::create(dir.join("program.c"))?;
    let mut header_file = fs::File::create(dir.join("program.h"))?;
    writeln!(program_file, "#include \"std.c\"")?;
    writeln!(program_file, "#include \"program.h\"\n")?;
    writeln!(program_file, "#include \"types.h\"\n")?;
    writeln!(header_file, "#include \"types.h\"\n")?;

    // Compile the program to c
    let mut used_types = vec![];
    for (cfg, deps) in cfgs {
        used_types.extend(&cfg.place_tys);
        compile_cfg(&mut program_file, &mut header_file, cfg, deps, type_map)?;
    }

    // Write the Makefile
    fs::write(dir.join("Makefile"), MAKEFILE)?;

    // Write the standard library
    write_std_lib(std_file, &mut type_file)?;

    // Write the types
    compile_tys(&mut type_file, used_types, type_map, type_map)?;
    writeln!(
        &mut type_file,
        "typedef struct {} unit;",
        type_name(&Type::unit(), type_map)
    )?;

    Ok(())
}

pub fn compile_tys<'a>(
    mut h: impl io::Write,
    anon_tys: impl IntoIterator<Item = &'a Type>,
    named_tys: impl IntoIterator<Item = (&'a String, &'a Type)>,
    type_map: &HashMap<String, Type>,
) -> io::Result<HashMap<Type, String>> {
    let mut remap = HashMap::new();

    // Create the types
    for anon_ty in anon_tys {
        compile_ty(&mut h, anon_ty, &mut remap, type_map)?;
    }
    for (name, ty) in named_tys {
        compile_ty(&mut h, ty, &mut remap, type_map)?;

        // Create the constructor
        if let Type::Enum(_) = ty {
            // let c_name = type_name(ty, type_map);
            // writeln!(h, "struct {c_name} *P_{name}(int disc, void *inner) {{",)?;
            // writeln!(
            //     h,
            //     "return (struct {c_name}) {{ .disc = disc, .inner = inner }};"
            // )?;
            // writeln!(h, "}}")?;

            let c_name = type_name(ty, type_map);
            writeln!(h, "struct {c_name} P_{name}(void *inner) {{",)?;
            writeln!(
                h,
                "return (struct {c_name}) {{ .disc = 0, .inner = inner }};"
            )?;
            writeln!(h, "}}")?;
        }
    }

    Ok(remap)
}

pub fn compile_ty(
    h: &mut impl io::Write,
    ty: &Type,
    remap: &mut HashMap<Type, String>,
    type_map: &HashMap<String, Type>,
) -> io::Result<()> {
    if let Some(_) = remap.get(ty) {
        return Ok(());
    }

    let name = match ty {
        Type::Tuple(t) => {
            let name = type_name(ty, type_map);
            remap.insert(ty.clone(), name.clone());

            for elem in &t.elems {
                compile_ty(h, elem, remap, type_map)?;
            }

            // Create the tuple type
            writeln!(h, "// {ty:?}")?;
            writeln!(h, "struct {name} {{")?;
            for (i, elem) in t.elems.iter().enumerate() {
                writeln!(h, "struct {} *e{i};", type_name(elem, type_map))?;
            }
            writeln!(h, "}};\n")?;

            // Create the conversion for that tuple type from its untyped version
            writeln!(
                h,
                "struct {name} from_tuple_base{name}(struct tuple_base{} base) {{",
                t.elems.len()
            )?;
            writeln!(h, "return *(struct {name} *) &base;")?;
            writeln!(h, "}}")?;

            name
        }
        Type::Enum(e) => {
            let name = type_name(ty, type_map);
            remap.insert(ty.clone(), name.clone());

            for variant in &e.variants {
                compile_ty(h, variant, remap, type_map)?;
            }

            // Create the enum struct
            writeln!(h, "// {ty:?}")?;
            writeln!(h, "struct {name} {{")?;
            writeln!(h, "int disc;")?;
            writeln!(h, "union {{")?;

            for (i, variant) in e.variants.iter().enumerate() {
                writeln!(h, "struct {} v{i};", type_name(variant, type_map))?;
            }

            writeln!(h, "}} *inner;")?;
            writeln!(h, "}};\n")?;

            name
        }
        Type::Named(n) => {
            let aliased_ty = type_map.get(n).unwrap();
            return compile_ty(h, aliased_ty, remap, type_map);
        }
    };

    // Create the allocation function for the type
    writeln!(h, "struct {name} *allocate_{name}(struct {name} val) {{")?;
    writeln!(h, "struct {name} *a = malloc(sizeof(val));")?;
    writeln!(h, "*a = val;")?;
    writeln!(h, "return a;")?;
    writeln!(h, "}}")?;

    Ok(())
}

fn type_name(ty: &Type, type_map: &HashMap<String, Type>) -> String {
    use convert_base::Convert;

    const ID_CHARS: &[u8] =
        "QWERTYUIOPASDFGHJKLZXCVBNMqwertyuiopasdfghjklzxcvbnm1234567890_".as_bytes();

    if let Type::Named(n) = ty {
        return type_name(type_map.get(n).unwrap(), type_map);
    }

    let mut h = DefaultHasher::new();
    ty.hash(&mut h);

    let encoded = Convert::new(64, ID_CHARS.len() as u64).convert::<u64, u8>(&[h.finish()]);
    let encoded_str: String =
        String::from_utf8(encoded.into_iter().map(|i| ID_CHARS[i as usize]).collect()).unwrap();

    format!("ty_{encoded_str}")
}

pub fn compile_cfg(
    mut c: impl io::Write,
    mut h: impl io::Write,
    cfg: &Cfg,
    deps: &DepGraph,
    type_map: &HashMap<String, Type>,
) -> io::Result<()> {
    let ret_ty = type_name(&cfg.place_tys[0], type_map);
    let alloced = if deps.nodes[0].allocated() { "*" } else { "" };
    write!(c, "struct {ret_ty} {alloced}P_{}(", cfg.name)?;
    write!(h, "struct {ret_ty} {alloced}P_{}(", cfg.name)?;

    for arg in 1..=cfg.arg_count {
        let arg_ty = type_name(&cfg.place_tys[arg], type_map);
        let alloced = if deps.nodes[arg].allocated() { "*" } else { "" };

        write!(c, "struct {arg_ty} {alloced}r{arg}")?;
        write!(h, "struct {arg_ty} {alloced}r{arg}")?;

        if arg != cfg.arg_count {
            write!(c, ", ")?;
            write!(h, ", ")?;
        }
    }
    writeln!(c, ") {{")?;
    writeln!(h, ");")?;

    for p in (cfg.arg_count + 1)..cfg.place_tys.len() {
        let alloced = if deps.nodes[p].allocated() { "*" } else { "" };

        writeln!(
            c,
            "struct {} {alloced}r{p};",
            type_name(&cfg.place_tys[p], type_map)
        )?;
    }

    let mut visited = vec![false; cfg.basic_blocks.len()];
    let mut bb_stack = vec![0];
    while let Some(bb) = bb_stack.pop() {
        visited[bb] = true;

        writeln!(c, "L_{bb}:")?;

        let block = &cfg.basic_blocks[bb];
        for stmnt in &block.stmnts {
            match stmnt {
                Statement::Assign(a) => {
                    let c_name = type_name(&cfg.place_tys[a.place], type_map);

                    write!(c, "r{} = ", a.place)?;

                    let mut closing_parens = 0;
                    if a.allocate {
                        write!(c, "allocate_{c_name}(")?;
                        closing_parens += 1;
                    }

                    match &a.value {
                        Value::Place(p) => write!(c, "r{p}")?,
                        Value::Call { func, .. } if func.0.as_str() == "invent" => {
                            write!(c, "invent()")?;
                        }
                        Value::Call { func, args } => {
                            match func.0.as_str() {
                                "tuple" => {
                                    closing_parens += 1;
                                    write!(c, "from_tuple_base{c_name}({func}{}(", args.len())?
                                }
                                name => write!(c, "P_{name}(")?,
                            };

                            for (i, arg) in args.iter().enumerate() {
                                write!(c, "r{arg}")?;

                                if i + 1 != args.len() {
                                    write!(c, ", ")?;
                                }
                            }
                            write!(c, ")")?;
                        }
                    }

                    write!(c, "{}", ")".repeat(closing_parens))?;
                    writeln!(c, ";")?;
                }
                Statement::Deallocate(r) => writeln!(c, "deallocate(r{r});")?,
                Statement::Dup(r) => writeln!(c, "dup(r{}, {});", r.place, r.count)?,
                Statement::Drop(r) => writeln!(c, "drop(r{}, {});", r.place, r.count)?,
                Statement::Nop => {}
            }
        }

        let succs = cfg.successors(bb);
        for &s in &succs {
            let succ = &cfg.basic_blocks[s];
            for phi in &succ.phi {
                if let Some(desired_place) = phi.opts.get(&bb) {
                    writeln!(c, "r{} = r{desired_place};", phi.place)?;
                }
            }
        }

        match block.terminator.as_ref().unwrap() {
            Terminator::Goto(next) => writeln!(c, "goto L_{next};")?,
            Terminator::Return(r) => writeln!(c, "return r{r};")?,
            Terminator::IfElse { cond, iff, elsee } => {
                writeln!(c, "if (r{cond}) goto L_{iff};")?;
                writeln!(c, "goto L_{elsee};")?;
            }
        }

        bb_stack.extend(succs.into_iter().filter(|s| !visited[*s]));
    }

    writeln!(c, "}}\n")?;

    Ok(())
}

pub fn write_std_lib(mut f: impl io::Write, mut types: impl io::Write) -> io::Result<()> {
    write!(f, "{STD_BASE}")?;

    for arg_count in 0..10 {
        // Create the tuple function for this arg count
        writeln!(types, "struct tuple_base{arg_count} {{")?;
        for i in 0..arg_count {
            writeln!(types, "void *e{i};")?;
        }
        writeln!(types, "}};")?;

        write!(f, "struct tuple_base{arg_count} tuple{arg_count}(")?;
        for i in 0..arg_count {
            write!(f, "void *r{i}")?;

            if i + 1 != arg_count {
                write!(f, ", ")?;
            }
        }
        writeln!(f, ") {{")?;

        write!(f, "return (struct tuple_base{arg_count}) {{ ")?;
        for i in 0..arg_count {
            write!(f, ".e{i} = r{i},")?;
        }
        writeln!(f, "}};")?;

        writeln!(f, "}}")?;
    }

    Ok(())
}
