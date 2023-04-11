use std::collections::HashMap;
use crate::ast::Term;

// TODO: Deal with variables in different rules (if `X` and `X` are in different rules, they aren't the same variable)

/// Inner representation for identifiers
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Debug)]
pub enum Identifier {
    Function(usize),
    Variable(usize),
}
#[derive(Default)]
pub struct IdentifierServer {
    variables_count: usize,
    functions_count: usize,
    ids_map: HashMap<Identifier, String>
}
impl IdentifierServer {
    /// Registers a new term, returning its identifier
    pub fn register(&mut self, t: &Term<String>) -> Identifier {
        match t {
            Term::Variable { symbol } => {
                let id = Identifier::Variable(self.variables_count);
                self.ids_map.insert(id, symbol.to_string());
                self.variables_count += 1;
                id
            }
            Term::Function {
                symbol,
                ..
            } => {
                let id = Identifier::Function(self.functions_count);
                self.ids_map.insert(id, symbol.to_string());
                self.functions_count += 1;
                id
            }
        }
    }

    /// Returns the name associated with the given identifier
    pub fn name_of(&self, id: &Identifier) -> &str {
        &self.ids_map[id]
    }
}