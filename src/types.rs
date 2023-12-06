use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Tuple(Tuple),
    Enum(Enum),
    Named(String),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Tuple {
    pub elems: Vec<Type>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Enum {
    pub variants: Vec<Type>,
}

impl Type {
    pub fn unit() -> Self {
        Self::Tuple(Tuple { elems: vec![] })
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tuple(t) => write!(f, "{t:?}"),
            Self::Enum(e) => write!(f, "{e:?}"),
            Self::Named(n) => write!(f, "{n}"),
        }
    }
}

impl fmt::Debug for Enum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, variant) in self.variants.iter().enumerate() {
            write!(f, "{variant:?}")?;

            if i + 1 != self.variants.len() {
                write!(f, " | ")?;
            }
        }
        write!(f, "]")?;

        Ok(())
    }
}

impl fmt::Debug for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for (i, elem) in self.elems.iter().enumerate() {
            write!(f, "{elem:?}")?;

            if i + 1 != self.elems.len() {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")?;

        Ok(())
    }
}
