use crate::ast::{Atom, InnerRule, Rule};
use crate::identifiers::{Identifier, IdentifierServer};

#[derive(Clone)]
pub enum Selection<T> {
    Premise(Atom<T>),
    Conclusion(Atom<T>),
}
impl TryFrom<(&Selection<Identifier>, &IdentifierServer)> for Selection<String> {
    type Error = ();
    fn try_from((s, id_server): (&Selection<Identifier>, &IdentifierServer)) -> Result<Self, Self::Error> {
        Ok(match s {
            Selection::Premise(a) => Selection::Premise(Atom::try_from((a, id_server))?),
            Selection::Conclusion(a) => Selection::Conclusion(Atom::try_from((a, id_server))?),
        })
    }
}

impl InnerRule {
    /// Resolution of r1 and r2
    /// r1 = |p| /\ q => r  (selected p)
    /// r2 = s /\ t => |c|  (selected c)
    /// if unfify(p, c) {
    ///     return asssigned(q /\ s /\ t => r, unify_context)
    /// }
    pub fn resolve(
        &self,
        other: &InnerRule,
        select: impl Fn(&InnerRule) -> Selection<Identifier>,
    ) -> Option<InnerRule> {
        match (select(self), select(other)) {
            (Selection::Premise(p), Selection::Conclusion(c))
            | (Selection::Conclusion(c), Selection::Premise(p)) => p.unify(&c).map(|bindings| {
                Rule {
                    conclusion: if self.conclusion == c {
                        other.conclusion.clone()
                    } else {
                        self.conclusion.clone()
                    },
                    premises: self
                        .premises
                        .iter()
                        .chain(&other.premises)
                        .cloned()
                        .filter(|a| *a != p && *a != c)
                        .collect::<Vec<_>>(),
                }
                .apply(&bindings)
            }),
            _ => None,
        }
    }
}
