use crate::ast::Term;
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
                }
                Term::Function { .. } => {
                    self.functions_count += 1;
                    Identifier::Function(self.functions_count - 1)
                }
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

/*
CONVERSION TRAITS
 */
mod conversion {
    use crate::ast::{Atom, InnerAtom, InnerRule, InnerTerm, Rule, Term};
    use crate::identifiers::IdentifierServer;

    // TERM
    impl From<(&Term<String>, &mut IdentifierServer)> for InnerTerm {
        fn from((t, id_server): (&Term<String>, &mut IdentifierServer)) -> Self {
            match t {
                Term::Variable { .. } => Term::Variable {
                    symbol: id_server.register(t),
                },
                Term::Function { parameters, .. } => {
                    let symbol = id_server.register(t);
                    let mut new_parameters = vec![];
                    for parameter in parameters {
                        new_parameters.push(Term::from((parameter, &mut *id_server)))
                    }
                    Term::Function {
                        symbol,
                        parameters: new_parameters,
                    }
                }
            }
        }
    }

    impl TryFrom<(&Term<String>, &IdentifierServer)> for InnerTerm {
        type Error = ();

        fn try_from(
            (t, id_server): (&Term<String>, &IdentifierServer),
        ) -> Result<Self, Self::Error> {
            Ok(match t {
                Term::Variable { symbol } => Term::Variable {
                    symbol: *id_server.id_of(symbol).ok_or(())?,
                },
                Term::Function { symbol, parameters } => {
                    let symbol = *id_server.id_of(symbol).ok_or(())?;
                    let mut new_parameters = vec![];
                    for parameter in parameters {
                        new_parameters.push(Term::try_from((parameter, id_server))?)
                    }
                    Term::Function {
                        symbol,
                        parameters: new_parameters,
                    }
                }
            })
        }
    }

    impl TryFrom<(&InnerTerm, &IdentifierServer)> for Term<String> {
        type Error = ();

        fn try_from((t, id_server): (&InnerTerm, &IdentifierServer)) -> Result<Self, Self::Error> {
            Ok(match t {
                Term::Variable { symbol } => Term::Variable {
                    symbol: id_server.name_of(symbol).ok_or(())?.clone(),
                },
                Term::Function { symbol, parameters } => {
                    let symbol = id_server.name_of(symbol).ok_or(())?.clone();
                    let mut new_parameters = vec![];
                    for parameter in parameters {
                        new_parameters.push(Term::try_from((parameter, id_server))?)
                    }
                    Term::Function {
                        symbol,
                        parameters: new_parameters,
                    }
                }
            })
        }
    }

    // ATOM
    impl From<(&Atom<String>, &mut IdentifierServer)> for InnerAtom {
        fn from((a, id_server): (&Atom<String>, &mut IdentifierServer)) -> Self {
            let Atom {
                symbol: _,
                parameters,
            } = a;
            let symbol = id_server.register(&Term::from(a.clone()));
            let mut new_parameters = vec![];
            for parameter in parameters {
                new_parameters.push(Term::from((parameter, &mut *id_server)))
            }
            Atom {
                symbol,
                parameters: new_parameters,
            }
        }
    }

    impl TryFrom<(&Atom<String>, &IdentifierServer)> for InnerAtom {
        type Error = ();

        fn try_from(
            (a, id_server): (&Atom<String>, &IdentifierServer),
        ) -> Result<Self, Self::Error> {
            let Atom { symbol, parameters } = a;
            let symbol = *id_server.id_of(symbol).ok_or(())?;
            let mut new_parameters = vec![];
            for parameter in parameters {
                new_parameters.push(Term::try_from((parameter, id_server))?)
            }
            Ok(Atom {
                symbol,
                parameters: new_parameters,
            })
        }
    }

    impl TryFrom<(&InnerAtom, &IdentifierServer)> for Atom<String> {
        type Error = ();

        fn try_from((a, id_server): (&InnerAtom, &IdentifierServer)) -> Result<Self, Self::Error> {
            let Atom { symbol, parameters } = a;
            let symbol = id_server.name_of(symbol).ok_or(())?.clone();
            let mut new_parameters = vec![];
            for parameter in parameters {
                new_parameters.push(Term::try_from((parameter, id_server))?)
            }
            Ok(Atom {
                symbol,
                parameters: new_parameters,
            })
        }
    }

    // RULE
    impl From<(&Rule<String>, &mut IdentifierServer)> for InnerRule {
        fn from((r, id_server): (&Rule<String>, &mut IdentifierServer)) -> Self {
            let Rule {
                premises,
                conclusion,
            } = r;
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

    impl TryFrom<(&Rule<String>, &IdentifierServer)> for InnerRule {
        type Error = ();

        fn try_from(
            (r, id_server): (&Rule<String>, &IdentifierServer),
        ) -> Result<Self, Self::Error> {
            let Rule {
                premises,
                conclusion,
            } = r;
            let conclusion = Atom::try_from((conclusion, id_server))?;
            let mut new_premises = vec![];
            for pre in premises {
                new_premises.push(Atom::try_from((pre, id_server))?)
            }
            Ok(Rule {
                conclusion,
                premises: new_premises,
            })
        }
    }

    impl TryFrom<(&InnerRule, &IdentifierServer)> for Rule<String> {
        type Error = ();

        fn try_from((r, id_server): (&InnerRule, &IdentifierServer)) -> Result<Self, Self::Error> {
            let Rule {
                premises,
                conclusion,
            } = r;
            let conclusion = Atom::try_from((conclusion, id_server))?;
            let mut new_premises = vec![];
            for pre in premises {
                new_premises.push(Atom::try_from((pre, id_server))?)
            }
            Ok(Rule {
                conclusion,
                premises: new_premises,
            })
        }
    }
}
