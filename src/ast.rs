//! AST module
//! High level representation of the constructs used in `.pif` files
use crate::Identifier;

pub type InnerTerm = Term<Identifier>;
/// Represents parsed terms
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Term<T> {
    Function { symbol: T, parameters: Vec<Term<T>> },
    Variable { symbol: T },
}
impl<T> Term<T> {
    pub fn symbol(&self) -> &T {
        match self {
            Term::Variable { symbol } => symbol,
            Term::Function { symbol, .. } => symbol,
        }
    }
}
/// Allows transformation of Atoms to Terms seamlessly
impl<T> From<Atom<T>> for Term<T> {
    fn from(value: Atom<T>) -> Term<T> {
        Term::Function {
            symbol: value.symbol,
            parameters: value.parameters,
        }
    }
}
impl<T: std::fmt::Display> std::fmt::Display for Term<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Function { symbol, parameters } if parameters.is_empty() => {
                write!(f, "{symbol}")
            }
            Term::Function { symbol, parameters } => {
                let parameters_pp = format_vec(parameters, ", ");
                write!(f, "{symbol}({parameters_pp})")
            }
            Term::Variable { symbol: value } => {
                write!(f, "{value}")
            }
        }
    }
}

pub type InnerAtom = Atom<Identifier>;
/// Represents parsed atoms, which are named lists of terms
/// Those are equivalent to Term::Function but necessary to avoid having variables
/// as top level objects
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Atom<T> {
    pub symbol: T,
    pub parameters: Vec<Term<T>>,
}
impl<T> TryFrom<Term<T>> for Atom<T> {
    type Error = ();

    fn try_from(value: Term<T>) -> Result<Self, Self::Error> {
        match value {
            Term::Function { symbol, parameters } => Ok(Atom { symbol, parameters }),
            _ => Err(()),
        }
    }
}
impl<T: std::fmt::Display> std::fmt::Display for Atom<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Atom { symbol, parameters } = self;
        if parameters.is_empty() {
            write!(f, "{symbol}")
        } else {
            let parameters_pp = format_vec(parameters, ", ");
            write!(f, "{symbol}({parameters_pp})")
        }
    }
}

pub type InnerRule = Rule<Identifier>;
/// Represents parsed rules as a list of premisses and the concluded atom
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Rule<T> {
    pub premises: Vec<Atom<T>>,
    pub conclusion: Atom<T>,
}
impl<T: std::fmt::Display> std::fmt::Display for Rule<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Rule {
            premises,
            conclusion,
        } = self;
        let premisses_pp = format_vec(premises, " /\\ ");
        write!(f, "{premisses_pp} => {conclusion}")
    }
}

/// Helper function to pretty print vectors
fn format_vec<T: std::fmt::Display>(v: &[T], sep: &str) -> String {
    v.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(sep)
}
