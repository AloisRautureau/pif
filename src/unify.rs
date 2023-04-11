use std::collections::HashMap;
use crate::ast::{InnerTerm, Term};
use crate::identifiers::Identifier;

#[derive(Default)]
pub struct UnificationContext<'a> {
    marker: usize,
    references: HashMap<Identifier, &'a InnerTerm>
}
impl<'a> UnificationContext<'a> {
    pub fn bind(&mut self, symbol: Identifier, term: &InnerTerm) {
        self.references.insert(symbol, term);
    }

    pub fn deref(&self, symbol: &Identifier) -> Option<&InnerTerm> {
        self.references.get(symbol).copied()
    }
}

impl InnerTerm {
    pub fn unify(&self, other: &InnerTerm) -> Result<(), ()> {
        fn inner(t1: &InnerTerm, t2: &InnerTerm, context: &mut UnificationContext) -> Result<(), ()> {
            // Finds leaves of terms `self` and `other`
            let (leaf1, leaf2) = (t1.leaf(context), t2.leaf(context));

            // If the leaves are equal, we can simply return
            if leaf1 == leaf2 {
                return Ok(())
            }

            // Otherwise, our actions depend on the types of `leaf1` and `leaf2`
            match (leaf1, leaf2) {
                (Term::Variable { symbol: x }, Term::Variable { symbol: y }) => {
                    context.bind(*x, leaf2);
                    Ok(())
                },
                (x @ Term::Variable { symbol: x_id}, f @ Term::Function { .. })
                | (f @ Term::Function { .. }, x @ Term::Variable { symbol: x_id}) => {
                    // TODO: if x in f => fail, else point x to f
                    if f.contains(x) {
                        Err(())
                    } else {
                        context.bind(*x_id, f);
                        Ok(())
                    }
                },
                (Term::Function { symbol: f, parameters: u}, Term::Function { symbol: g, parameters: v }) => {
                    if f == g && u.len() == v.len() {
                        let _ = u.iter().zip(v.iter())
                            .map(|(x, y)| x.unify(y))
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(())
                    } else {
                        Err(())
                    }
                }
            }
        }

        let mut unification_context = UnificationContext::default();
        inner(self, other, &mut unification_context)
    }

    /// Returns the leaf term of `self`
    fn leaf(&self, context: &UnificationContext) -> &InnerTerm {
        match self {
            Term::Function { .. } => self,
            Term::Variable { symbol } => if let Some(bound_term) = context.deref(symbol) {
                bound_term.leaf(context)
            } else {
                self
            }
        }
    }

    /// Checks if this term contains `t`
    fn contains(&self, t: &InnerTerm) -> bool {
        todo!()
    }
}