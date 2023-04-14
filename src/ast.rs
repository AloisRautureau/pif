//! AST module
//! High level representation of the constructs used in `.pif` files
use crate::Identifier;
use std::collections::HashMap;
use std::hash::Hash;

pub type InnerTerm = Term<Identifier>;
/// Represents parsed terms
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Term<T> {
    Function { symbol: T, parameters: Vec<Term<T>> },
    Variable { symbol: T },
}

impl<T: Clone + Hash + Eq + PartialEq> Term<T> {
    pub fn symbol(&self) -> &T {
        match self {
            Term::Variable { symbol } => symbol,
            Term::Function { symbol, .. } => symbol,
        }
    }
    /// Applies a valuation of the variables to this rule
    pub fn apply(&self, bindings: &HashMap<Term<T>, Term<T>>) -> Term<T> {
        if let Some(binding) = bindings.get(self) {
            binding.clone()
        } else {
            self.clone()
        }
    }

    pub fn is_variable(&self) -> bool {
        matches!(self, Term::Variable { .. })
    }

    pub fn contains_variable(&self, variable: &Term<T>) -> bool {
        let Term::Variable {symbol} = variable else { panic!("Expected variable")};
        match self {
            Term::Function { parameters, .. } => {
                parameters.iter().any(|t| t.contains_variable(variable))
            }
            Term::Variable { symbol: v } => v == symbol,
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
impl<T: Clone + Hash + Eq + PartialEq> Atom<T> {
    /// Applies a valuation of the variables to this rule
    pub fn apply(&self, bindings: &HashMap<Term<T>, Term<T>>) -> Atom<T> {
        Atom {
            symbol: self.symbol.clone(),
            parameters: self
                .parameters
                .iter()
                .cloned()
                .map(|t| t.apply(bindings))
                .collect(),
        }
    }

    pub fn contains_variable(&self, variable: &Term<T>) -> bool {
        self.parameters
            .iter()
            .any(|t| t.contains_variable(variable))
    }

    // Checks if the given atom is like Symbol(...)
    pub fn is_symbol(&self, symbol: T) -> bool {
        self.symbol == symbol
    }

    /// Checks if the given atom is like Symbol(X) where X is a variable
    pub fn is_smth_of_variable(&self) -> bool {
        self.parameters.len() == 1 && self.parameters[0].is_variable()
    }
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
impl<T: Clone + Hash + Eq + PartialEq> Rule<T> {
    /// Applies a valuation of the variables to this rule
    pub fn apply(&self, bindings: &HashMap<Term<T>, Term<T>>) -> Rule<T> {
        Rule {
            conclusion: self.conclusion.apply(bindings),
            premises: self
                .premises
                .iter()
                .cloned()
                .map(|a| a.apply(bindings))
                .collect(),
        }
    }
}
impl<T: std::fmt::Display> std::fmt::Display for Rule<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Rule {
            premises,
            conclusion,
        } = self;
        if premises.is_empty() {
            write!(f, "{conclusion}")
        } else {
            let premises_pp = format_vec(premises, " /\\ ");
            write!(f, "{premises_pp} => {conclusion}")
        }
    }
}

/// Helper function to pretty print vectors
fn format_vec<T: std::fmt::Display>(v: &[T], sep: &str) -> String {
    v.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(sep)
}
