use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs,
    hash::{Hash, Hasher},
    io,
    io::Write,
    path::Path,
};

use crate::{
    cfg::{Cfg, Statement, Terminator, Value},
    types::Type,
};

const MAKEFILE: &str = include_str!("Makefile");
const STD_BASE: &str = include_str!("std_base.c");

pub fn compile_module_to_dir<'a>(
    dir: impl AsRef<Path>,
    cfgs: impl IntoIterator<Item = &'a Cfg>,
    type_map: &HashMap<String, Type>,
) -> io::Result<()> {
    // Create the build directorya
    let dir = dir.as_ref();
    fs::create_dir_all(dir)?;

    // Compile the program to c
    let mut program_file = fs::File::create(dir.join("program.c"))?;
    let mut header_file = fs::File::create(dir.join("program.h"))?;
    writeln!(program_file, "#include \"std.c\"")?;
    writeln!(program_file, "#include \"program.h\"\n")?;
    writeln!(program_file, "#include \"types.h\"\n")?;

    let mut used_types = vec![];
    for cfg in cfgs {
        used_types.extend(&cfg.place_tys);
        compile_cfg(&mut program_file, &mut header_file, cfg)?;
    }

    // Write the Makefile
    fs::write(dir.join("Makefile"), MAKEFILE)?;

    // Write the standard library
    let std_file = fs::File::create(dir.join("std.c"))?;
    write_std_lib(std_file)?;

    // Write the types
    let type_file = fs::File::create(dir.join("types.h"))?;
    compile_tys(type_file, used_types, type_map, type_map)?;

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
            //     "return (struct {c_name}) {{ .disc = disc, .inner = inner }}"
            // )?;
            // writeln!(h, "}}")?;

            writeln!(h, "void *P_{name}(void *inner) {{ return (void *) 0; }}")?;
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

    match ty {
        Type::Tuple(t) => {
            let name = type_name(ty, type_map);
            remap.insert(ty.clone(), name.clone());

            for elem in &t.elems {
                compile_ty(h, elem, remap, type_map)?;
            }

            writeln!(h, "// {ty:?}")?;
            writeln!(h, "struct {name} {{")?;
            for (i, elem) in t.elems.iter().enumerate() {
                writeln!(h, "struct {} *e{i};", type_name(elem, type_map))?;
            }
            writeln!(h, "}};\n")?;
        }
        Type::Enum(e) => {
            let name = type_name(ty, type_map);
            remap.insert(ty.clone(), name.clone());

            for variant in &e.variants {
                compile_ty(h, variant, remap, type_map)?;
            }

            writeln!(h, "// {ty:?}")?;
            writeln!(h, "struct {name} {{")?;
            writeln!(h, "int disc;")?;
            writeln!(h, "union {{")?;

            for (i, variant) in e.variants.iter().enumerate() {
                writeln!(h, "struct {} v{i};", type_name(variant, type_map))?;
            }

            writeln!(h, "}} *inner;")?;
            writeln!(h, "}};\n")?;
        }
        Type::Named(n) => {
            let aliased_ty = type_map.get(n).unwrap();
            compile_ty(h, aliased_ty, remap, type_map)?;
        }
    }

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

pub fn compile_cfg(mut c: impl io::Write, mut h: impl io::Write, cfg: &Cfg) -> io::Result<()> {
    write!(c, "void *P_{}(", cfg.name)?;
    write!(h, "void *P_{}(", cfg.name)?;
    for arg in 1..=cfg.arg_count {
        write!(c, "void *r{arg}")?;
        write!(h, "void *r{arg}")?;

        if arg != cfg.arg_count {
            write!(c, ", ")?;
            write!(h, ", ")?;
        }
    }
    writeln!(c, ") {{")?;
    writeln!(h, ");")?;

    for place in (cfg.arg_count + 1)..cfg.place_tys.len() {
        writeln!(c, "void *r{place};")?;
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
                    write!(c, "r{} = ", a.place)?;

                    if a.allocate {
                        write!(c, "allocate(")?;
                    }

                    match &a.value {
                        Value::Place(p) => write!(c, "r{p}")?,
                        Value::Call { func, args } => {
                            match func.0.as_str() {
                                "tuple" | "invent" | "print" => write!(c, "{func}{}(", args.len())?,
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

                    if a.allocate {
                        write!(c, ")")?;
                    }
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

pub fn write_std_lib(mut f: impl io::Write) -> io::Result<()> {
    write!(f, "{STD_BASE}")?;

    for arg_count in 0..10 {
        for name in ["tuple", "invent", "print"] {
            write!(f, "void *{name}{arg_count}(")?;
            for i in 0..arg_count {
                write!(f, "void *r{i}")?;

                if i + 1 != arg_count {
                    write!(f, ", ")?;
                }
            }
            writeln!(f, ") {{")?;

            writeln!(f, "return (void *) 0;")?;

            writeln!(f, "}}")?;
        }
    }

    Ok(())
}
