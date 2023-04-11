use std::collections::HashMap;
use crate::ast::{Atom, InnerAtom, InnerRule, InnerTerm, Term};
use crate::identifiers::Identifier;

#[derive(Default)]
pub struct UnificationContext {
    marker: usize,
    references: HashMap<Identifier, InnerTerm>
}
impl UnificationContext {
    pub fn bind(&mut self, symbol: Identifier, term: InnerTerm) {
        self.references.insert(symbol, term);
    }

    pub fn deref<'a>(&'a self, mut symbol: &'a Identifier) -> Option<&InnerTerm> {
        loop {
            match self.references.get(symbol) {
                Some(Term::Variable { symbol: s }) => symbol = s,
                x => break x
            }
        }
    }
}

impl InnerRule {
    pub fn assign(&self, values: &Vec<&InnerTerm>) -> Result<InnerAtom, ()> {
        fn unify(t1: &InnerTerm, t2: &InnerTerm, context: &mut UnificationContext) -> Result<(), ()> {
            // Finds leaves of terms `self` and `other`
            let (leaf1, leaf2) = (t1.leaf(context), t2.leaf(context));

            // If the leaves are equal, we can simply return
            if leaf1 == leaf2 {
                return Ok(())
            }

            // Otherwise, our actions depend on the types of `leaf1` and `leaf2`
            match (&leaf1, &leaf2) {
                (Term::Variable { symbol: x }, Term::Variable { .. }) => {
                    context.bind(*x, leaf2.clone());
                    Ok(())
                },
                (x @ Term::Variable { symbol: x_id}, f @ Term::Function { .. })
                | (f @ Term::Function { .. }, x @ Term::Variable { symbol: x_id}) => {
                    if f.contains(&x) {
                        Err(())
                    } else {
                        context.bind(*x_id, f.clone());
                        Ok(())
                    }
                },
                (Term::Function { symbol: f, parameters: u}, Term::Function { symbol: g, parameters: v }) => {
                    if f == g && u.len() == v.len() {
                        let _ = u.iter().zip(v.iter())
                            .map(|(x, y)| unify(x, y, &mut *context))
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(())
                    } else {
                        Err(())
                    }
                }
            }
        }

        let mut unification_context = UnificationContext::default();
        for (pre, val) in self.premises.iter().zip(values) {
            unify(&Term::from(pre.clone()), val, &mut unification_context)?
        }
        Ok(Atom::try_from(Term::from(self.conclusion.clone()).apply(&unification_context)).unwrap())
    }
}

impl InnerTerm {
    /// Returns the leaf term of `self`
    pub fn leaf(&self, context: &UnificationContext) -> InnerTerm {
        match self {
            Term::Function { .. } => self.clone(),
            Term::Variable { symbol } => if let Some(bound_term) = context.deref(symbol) {
                bound_term.leaf(context)
            } else {
                self.clone()
            }
        }
    }

    /// Checks if this term contains variable `t`
    pub fn contains(&self, t: &InnerTerm) -> bool {
        if self == t {
            return true
        }

        if let Term::Function { parameters, .. } = self {
            for param in parameters {
                if param.contains(t) { return true }
            }
        }

        false
    }

    /// Applies a context to a term, setting its variables to their associated valued
    pub fn apply(&self, context: &UnificationContext) -> InnerTerm {
        match self {
            Term::Variable {
                symbol
            } => if let Some(t) = context.deref(symbol) {
                t.clone()
            } else {
                self.clone()
            },
            Term::Function {
                symbol,
                parameters,
            } => Term::Function {
                symbol: *symbol,
                parameters: parameters.iter().map(|t| t.apply(context)).collect(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_test() {
        let test_var_term = Term::Variable { symbol: Identifier::Variable(0) };
        let test_fun_term = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![
                Term::Variable { symbol: Identifier::Variable(1) },
                Term::Function {
                    symbol: Identifier::Function(1),
                    parameters: vec![Term::Variable { symbol: Identifier::Variable(0) }],
                },
                Term::Variable { symbol: Identifier::Variable(3) },
            ]
        };

        assert!(test_var_term.contains(&Term::Variable { symbol: Identifier::Variable(0) }));
        assert!(!test_var_term.contains(&Term::Variable { symbol: Identifier::Variable(12) }));

        assert!(test_fun_term.contains(&Term::Variable { symbol: Identifier::Variable(1) }));
        assert!(test_fun_term.contains(&Term::Variable { symbol: Identifier::Variable(0) }));
        assert!(test_fun_term.contains(&Term::Variable { symbol: Identifier::Variable(3) }));
        assert!(!test_fun_term.contains(&Term::Variable { symbol: Identifier::Variable(12) }));
    }

    #[test]
    fn apply_test() {
        let test_var_term = Term::Variable { symbol: Identifier::Variable(0) };
        let test_fun_term = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![
                Term::Variable { symbol: Identifier::Variable(1) },
                Term::Function {
                    symbol: Identifier::Function(1),
                    parameters: vec![Term::Variable { symbol: Identifier::Variable(0) }],
                },
                Term::Variable { symbol: Identifier::Variable(3) },
            ]
        };

        let context = UnificationContext {
            references: HashMap::from([
            (Identifier::Variable(0), Term::Function { symbol: Identifier::Function(2), parameters: vec![] }),
        (Identifier::Variable(1), Term::Variable { symbol: Identifier::Variable(0) }),
        (Identifier::Variable(2), Term::Function { symbol: Identifier::Function(4), parameters: vec![] }),
        (Identifier::Variable(3), Term::Function { symbol: Identifier::Function(5), parameters: vec![] }),
        ]),
            marker: 0
        };

        let applied_var = test_var_term.apply(&context);
        let applied_fun = test_fun_term.apply(&context);
        assert_eq!(applied_var, Term::Function { symbol: Identifier::Function(2), parameters: vec![] });
        assert_eq!(applied_fun, Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![
                Term::Function { symbol: Identifier::Function(2), parameters: vec![] },
                Term::Function {
                    symbol: Identifier::Function(1),
                    parameters: vec![Term::Function { symbol: Identifier::Function(2), parameters: vec![] }],
                },
                Term::Function { symbol: Identifier::Function(5), parameters: vec![] },
            ]
        })
    }
}