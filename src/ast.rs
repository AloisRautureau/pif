//! AST module
//! High level representation of the constructs used in `.pif` files

/// Represents parsed terms
#[derive(Debug, Clone)]
pub enum Term<T> {
    Application {
        symbol: String,
        terms: Vec<Term<T>>,
    },
    Variable {
        value: T,
    }
}
impl<T: std::fmt::Display> std::fmt::Display for Term<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Application {
                symbol,
                terms
            } => {
                let terms_pp = format_vec(terms, ", ");
                write!(f, "{symbol}({terms_pp})")
            },
            Term::Variable {
                value
            } => {
                write!(f, "{value}")
            }
        }
    }
}

/// Represents parsed atoms, which are named lists of terms
#[derive(Debug, Clone)]
pub struct Atom<T> {
    pub symbol: String,
    pub terms: Vec<Term<T>>,
}
impl<T: std::fmt::Display> std::fmt::Display for Atom<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Atom { symbol, terms } = self;
        let terms_pp = format_vec(terms, ", ");
        write!(f, "{symbol}({terms_pp})")
    }
}

/// Represents parsed rules as a list of premisses and the concluded atom
#[derive(Debug, Clone)]
pub struct Rule<T> {
    pub premisses: Vec<Atom<T>>,
    pub conclusion: Atom<T>,
}
impl<T: std::fmt::Display> std::fmt::Display for Rule<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Rule { premisses, conclusion } = self;
        let premisses_pp = format_vec(premisses, " /\\");
        write!(f, "{premisses_pp} => {conclusion}")
    }
}

/// Helper function to pretty print vectors
fn format_vec<T: std::fmt::Display>(v: &Vec<T>, sep: &str) -> String {
    v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(sep)
}