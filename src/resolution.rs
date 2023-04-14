use crate::ast::{Atom, InnerRule, Rule};
use crate::identifiers::{Identifier, IdentifierServer};

#[derive(Clone)]
pub enum Selection<T> {
    Premise(Atom<T>, usize),
    Conclusion(Atom<T>),
}
impl TryFrom<(&Selection<Identifier>, &IdentifierServer)> for Selection<String> {
    type Error = ();
    fn try_from(
        (s, id_server): (&Selection<Identifier>, &IdentifierServer),
    ) -> Result<Self, Self::Error> {
        Ok(match s {
            Selection::Premise(a, i) => Selection::Premise(Atom::try_from((a, id_server))?, *i),
            Selection::Conclusion(a) => Selection::Conclusion(Atom::try_from((a, id_server))?),
        })
    }
}

impl InnerRule {
    /// Resolution of r1 and r2
    /// r1 = |p| /\ q => r  (selected p)
    /// r2 = s /\ t => |c|  (selected c)
    /// if unfify(p, c) {
    ///     (q /\ s /\ t => r).asssigned(unify_context)
    ///     - delete every Att(X) from q /\ s /\ t where X is not in r
    ///     return (q /\ s /\ t => r)
    /// }
    pub fn resolve(
        &self,
        other: &InnerRule,
        select: impl Fn(&InnerRule) -> Selection<Identifier>,
        keep: impl Fn(&Atom<Identifier>, &Atom<Identifier>) -> bool,
    ) -> Option<InnerRule> {
        match (select(self), select(other)) {
            (Selection::Premise(p, i), Selection::Conclusion(c)) => p.unify(&c).map(|bindings| {
                let mut premises = self.premises.clone();
                premises.remove(i);
                premises.append(&mut other.premises.clone());
                let mut rule = Rule {
                    conclusion: self.conclusion.clone(),
                    premises: premises,
                };
                rule = rule.apply(&bindings);
                rule.premises = rule
                    .premises
                    .into_iter()
                    .filter(|p| keep(p, &rule.conclusion))
                    .collect();
                rule
            }),
            (Selection::Conclusion(_), Selection::Premise(_, _)) => {
                other.resolve(self, select, keep)
            }

            _ => None,
        }
    }
}
