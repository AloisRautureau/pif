use std::collections::HashMap;
use crate::ast::Term;

// TODO: Deal with variables in different rules (if `X` and `X` are in different rules, they aren't the same variable)

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
    ids_map: HashMap<Identifier, String>,
    names_map: HashMap<String, Identifier>,
}
impl IdentifierServer {
    /// Registers a new term, returning its identifier
    pub fn register(&mut self, t: &Term<String>) -> Identifier {
        let symbol = t.symbol();
        if let Some(id) = self.names_map.get(symbol) {
            *id
        } else {
            let id = match t {
                Term::Variable { .. } => {
                    self.variables_count += 1;
                    Identifier::Variable(self.variables_count - 1)
                },
                Term::Function { .. } => {
                    self.functions_count += 1;
                    Identifier::Function(self.functions_count - 1)
                },
            };

            self.ids_map.insert(id, symbol.to_string());
            self.names_map.insert(symbol.to_string(), id);
            id
        }
    }

    /// Returns the name associated with the given identifier
    pub fn name_of(&self, id: &Identifier) -> Option<&String> {
        self.ids_map.get(id)
    }

    pub fn id_of(&self, name: &str) -> Option<&Identifier> {
        self.names_map.get(name)
    }
}