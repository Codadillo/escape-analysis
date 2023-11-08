use std::fmt;

use super::{BasicBlock, Cfg, Statement, Terminator, Value};

impl fmt::Debug for Cfg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cfg",)?;
        fmt_arglist(f, 1..=self.arg_count)?;
        writeln!(f, ":")?;

        for (i, bb) in self.basic_blocks.iter().enumerate() {
            writeln!(f, "{i}: {bb:#?}")?;
        }

        Ok(())
    }
}

impl fmt::Debug for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;

        for phi in &self.phi {
            write!(f, "\tlet _{} = ϕ", phi.place)?;
            fmt_named_arglist(f, phi.opts.clone())?;
            writeln!(f, ";")?;
        }

        for stmnt in &self.stmnts {
            write!(f, "\t")?;
            match stmnt {
                Statement::Assign(a) => write!(f, "let _{} = {:?}", a.place, a.value)?,
            }
            writeln!(f, ";")?;
        }

        if let Some(terminator) = &self.terminator {
            writeln!(f, "\t{:?}", terminator)?;
        } else {
            writeln!(f, "\tdeadend")?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Place(p) => write!(f, "_{p}"),
            Value::Call { func, args } => {
                write!(f, "{}", func.0)?;
                fmt_arglist(f, args.clone())
            }
        }
    }
}

impl fmt::Debug for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Goto(bb) => write!(f, "goto -> {bb}"),
            Self::Return => write!(f, "return _0"),
            Self::IfElse { cond, iff, elsee } => {
                write!(f, "goto -> if _{cond} {{ {iff} }} else {{ {elsee} }}")
            }
        }
    }
}

fn fmt_arglist(f: &mut fmt::Formatter<'_>, args: impl IntoIterator<Item = usize>) -> fmt::Result {
    write!(f, "(")?;

    let mut args = args.into_iter().peekable();
    while let Some(arg) = args.next() {
        write!(f, "_{arg}")?;
        if args.peek().is_some() {
            write!(f, ", ")?;
        }
    }

    write!(f, ")")
}

fn fmt_named_arglist<N: fmt::Debug>(
    f: &mut fmt::Formatter<'_>,
    args: impl IntoIterator<Item = (N, usize)>,
) -> fmt::Result {
    write!(f, "(")?;

    let mut args = args.into_iter().peekable();
    while let Some((name, arg)) = args.next() {
        write!(f, "{name:?}: _{arg}")?;
        if args.peek().is_some() {
            write!(f, ", ")?;
        }
    }

    write!(f, ")")
}
