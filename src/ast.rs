//! AST module
//! High level representation of the constructs used in `.pif` files
use crate::{Identifier, IdentifierServer};

pub type InnerTerm = Term<Identifier>;
/// Represents parsed terms
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Term<T> {
    Function {
        symbol: T,
        parameters: Vec<Term<T>>,
    },
    Variable {
        symbol: T,
    }
}
/// Allows transformation of Atoms to Terms seamlessly
impl<T> From<Atom<T>> for Term<T> {
    fn from(value: Atom<T>) -> Term<T> {
        Term::Function {
            symbol: value.symbol,
            parameters: value.parameters
        }
    }
}
impl From<(&Term<String>, &mut IdentifierServer)> for InnerTerm {
    fn from((t, id_server): (&Term<String>, &mut IdentifierServer)) -> Self {
        match t {
            Term::Variable { .. } => Term::Variable {
                symbol: id_server.register(t)
            },
            Term::Function { parameters, .. } => {
                let symbol = id_server.register(t);
                let mut new_parameters = vec![];
                for parameter in parameters {
                    new_parameters.push(Term::from((parameter, &mut *id_server)))
                }
                Term::Function {
                    symbol,
                    parameters: new_parameters
                }
            }
        }
    }
}
impl<T: std::fmt::Display> std::fmt::Display for Term<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Function {
                symbol,
                parameters: terms
            } => {
                let terms_pp = format_vec(terms, ", ");
                write!(f, "{symbol}({terms_pp})")
            },
            Term::Variable {
                symbol: value
            } => {
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
            Term::Function {
                symbol,
                parameters
            } => Ok(Atom {
                symbol,
                parameters
            }),
            _ => Err(())
        }
    }
}
impl From<(&Atom<String>, &mut IdentifierServer)> for InnerAtom {
    fn from((a, id_server): (&Atom<String>, &mut IdentifierServer)) -> Self {
        let Atom { symbol: _, parameters } = a;
        let symbol = id_server.register(&Term::from(a.clone()));
        let mut new_parameters = vec![];
        for parameter in parameters {
            new_parameters.push(Term::from((parameter, &mut *id_server)))
        }
        Atom {
            symbol,
            parameters: new_parameters
        }
    }
}
impl<T: std::fmt::Display> std::fmt::Display for Atom<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Atom { symbol, parameters: terms } = self;
        let terms_pp = format_vec(terms, ", ");
        write!(f, "{symbol}({terms_pp})")
    }
}

pub type InnerRule = Rule<Identifier>;
/// Represents parsed rules as a list of premisses and the concluded atom
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Rule<T> {
    pub premises: Vec<Atom<T>>,
    pub conclusion: Atom<T>,
}
impl From<(&Rule<String>, &mut IdentifierServer)> for InnerRule {
    fn from((r, id_server): (&Rule<String>, &mut IdentifierServer)) -> Self {
        let Rule { premises, conclusion } = r;
        let conclusion = Atom::from((conclusion, &mut *id_server));
        let mut new_premises = vec![];
        for pre in premises {
            new_premises.push(Atom::from((pre, &mut *id_server)))
        }
        Rule {
            conclusion,
            premises: new_premises,
        }
    }
}
impl<T: std::fmt::Display> std::fmt::Display for Rule<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Rule { premises, conclusion } = self;
        let premisses_pp = format_vec(premises, " /\\");
        write!(f, "{premisses_pp} => {conclusion}")
    }
}

/// Helper function to pretty print vectors
fn format_vec<T: std::fmt::Display>(v: &Vec<T>, sep: &str) -> String {
    v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(sep)
}