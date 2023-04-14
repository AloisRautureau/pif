use crate::ast::{Atom, InnerAtom, InnerRule, InnerTerm, Rule, Term};
use rustc_hash::FxHashMap;
use std::collections::HashMap;

/// Inner representation for identifiers
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Debug)]
pub enum Identifier {
    Function(usize),
    Variable(usize),
}
#[derive(Default, Debug)]
pub struct IdentifierServer {
    variables_count: usize,
    functions_count: usize,
    ids_map: FxHashMap<Identifier, String>,
    names_map: FxHashMap<String, Identifier>,
}
impl IdentifierServer {
    /// Registers a new term, returning its identifier
    pub fn register_function(&mut self, symbol: &str) -> Identifier {
        if let Some(identifier) = self.names_map.get(symbol) {
            *identifier
        } else {
            let identifier = Identifier::Function(self.functions_count);
            self.functions_count += 1;
            self.ids_map.insert(identifier, symbol.to_string());
            self.names_map.insert(symbol.to_string(), identifier);
            identifier
        }
    }

    pub fn register_variable(&mut self) -> Identifier {
        let id = self.variables_count;
        let identifier = Identifier::Variable(id);
        let symbol = String::from("VAR") + &id.to_string();
        self.ids_map.insert(identifier, symbol.clone());
        self.names_map.insert(symbol, identifier);
        self.variables_count += 1;
        identifier
    }

    /// Returns the name associated with the given identifier
    pub fn name_of(&self, id: &Identifier) -> Option<String> {
        match id {
            Identifier::Function(_) => self.ids_map.get(id).cloned(),
            Identifier::Variable(i) => Some(String::from("VAR") + &i.to_string()),
        }
    }

    pub fn id_of(&self, name: &str) -> Option<&Identifier> {
        self.names_map.get(name)
    }
}

impl Term<String> {
    pub fn to_inner(
        &self,
        id_server: &mut IdentifierServer,
        bindings: &mut HashMap<String, Identifier>,
    ) -> InnerTerm {
        match self {
            Term::Variable { symbol } => {
                let identifier = if let Some(identifier) = bindings.get(symbol) {
                    *identifier
                } else {
                    let identifier = id_server.register_variable();
                    bindings.insert(symbol.clone(), identifier);
                    identifier
                };
                Term::Variable { symbol: identifier }
            }
            Term::Function { symbol, parameters } => Term::Function {
                symbol: id_server.register_function(symbol),
                parameters: parameters
                    .iter()
                    .cloned()
                    .map(|t| t.to_inner(id_server, bindings))
                    .collect(),
            },
        }
    }
}
impl InnerTerm {
    pub fn to_string(&self, id_server: &IdentifierServer) -> Term<String> {
        match self {
            Term::Variable { symbol } => Term::Variable {
                symbol: id_server.name_of(symbol).unwrap(),
            },
            Term::Function { symbol, parameters } => Term::Function {
                symbol: id_server.name_of(symbol).unwrap(),
                parameters: parameters
                    .iter()
                    .cloned()
                    .map(|t| t.to_string(id_server))
                    .collect(),
            },
        }
    }

    pub fn make_fresh(
        &self,
        id_server: &mut IdentifierServer,
        bindings: &mut HashMap<Identifier, Identifier>,
    ) -> InnerTerm {
        match self {
            Term::Variable { symbol } => {
                let identifier = if let Some(identifier) = bindings.get(symbol) {
                    *identifier
                } else {
                    let identifier = id_server.register_variable();
                    bindings.insert(*symbol, identifier);
                    identifier
                };
                Term::Variable { symbol: identifier }
            }
            Term::Function { symbol, parameters } => Term::Function {
                symbol: *symbol,
                parameters: parameters
                    .iter()
                    .cloned()
                    .map(|t| t.make_fresh(id_server, bindings))
                    .collect(),
            },
        }
    }
}

impl Atom<String> {
    pub fn to_inner(
        &self,
        id_server: &mut IdentifierServer,
        bindings: &mut HashMap<String, Identifier>,
    ) -> InnerAtom {
        Atom {
            symbol: id_server.register_function(&self.symbol),
            parameters: self
                .parameters
                .iter()
                .map(|t| t.to_inner(id_server, bindings))
                .collect(),
        }
    }
}
impl InnerAtom {
    pub fn to_string(&self, id_server: &IdentifierServer) -> Atom<String> {
        Atom {
            symbol: id_server.name_of(&self.symbol).unwrap(),
            parameters: self
                .parameters
                .iter()
                .map(|t| t.to_string(id_server))
                .collect(),
        }
    }

    pub fn make_fresh(
        &self,
        id_server: &mut IdentifierServer,
        bindings: &mut HashMap<Identifier, Identifier>,
    ) -> InnerAtom {
        Atom {
            symbol: self.symbol,
            parameters: self
                .parameters
                .iter()
                .map(|t| t.make_fresh(id_server, bindings))
                .collect(),
        }
    }
}

impl Rule<String> {
    pub fn to_inner(&self, id_server: &mut IdentifierServer) -> InnerRule {
        let mut bindings = HashMap::new();
        Rule {
            conclusion: self.conclusion.to_inner(id_server, &mut bindings),
            premises: self
                .premises
                .iter()
                .map(|a| a.to_inner(id_server, &mut bindings))
                .collect(),
        }
    }
}
impl InnerRule {
    pub fn to_string(&self, id_server: &IdentifierServer) -> Rule<String> {
        Rule {
            conclusion: self.conclusion.to_string(id_server),
            premises: self
                .premises
                .iter()
                .map(|a| a.to_string(id_server))
                .collect(),
        }
    }

    pub fn make_fresh(&self, id_server: &mut IdentifierServer) -> InnerRule {
        let mut bindings = HashMap::new();
        Rule {
            conclusion: self.conclusion.make_fresh(id_server, &mut bindings),
            premises: self
                .premises
                .iter()
                .map(|a| a.make_fresh(id_server, &mut bindings))
                .collect(),
        }
    }
}
