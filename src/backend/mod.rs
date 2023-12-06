use std::{fs, io, io::Write, path::Path};

use crate::cfg::{Cfg, Statement, Terminator, Value};

const MAKEFILE: &str = include_str!("Makefile");
const STD_BASE: &str = include_str!("std_base.c");

pub fn compile_cfgs_to_dir<'a>(
    dir: impl AsRef<Path>,
    cfgs: impl IntoIterator<Item = &'a Cfg>,
) -> io::Result<()> {
    // Create the build directorya
    let dir = dir.as_ref();
    fs::create_dir_all(dir)?;

    // Compile the program to c
    let mut program_file = fs::File::create(dir.join("program.c"))?;
    let mut header_file = fs::File::create(dir.join("program.h"))?;
    writeln!(program_file, "#include \"std.c\"")?;
    writeln!(program_file, "#include \"program.h\"\n")?;

    for cfg in cfgs {
        compile_cfg_to_c(&mut program_file, &mut header_file, cfg)?;
    }

    // Write the Makefile
    fs::write(dir.join("Makefile"), MAKEFILE)?;

    // Write the standard library
    let std_file = fs::File::create(dir.join("std.c"))?;
    write_std_lib(std_file)?;

    Ok(())
}

pub fn compile_cfg_to_c(mut c: impl io::Write, mut h: impl io::Write, cfg: &Cfg) -> io::Result<()> {
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

    for place in (cfg.arg_count + 1)..cfg.place_count {
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
