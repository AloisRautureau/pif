use crate::{
    ast::{Atom, InnerRule, Rule},
    identifiers::Identifier,
};

pub enum Selection {
    Premise(usize),
    Conclusion,
}

impl InnerRule {
    /// Selection of an atom in a rule
    /// (n in 0..+inf)
    /// Input : r = A_1 /\ ... /\ A_n => B
    /// Output : (Premise, i) or (Conclusion, None)
    /// - (Premise, i) if A_i is selected
    /// - (Conclusion, None) if B is selected
    /// When we find Att(t) with t not a variable, we select it
    /// else we select the conclusion
    pub fn select(&self) -> Selection {
        for (i, premise) in self.premises.iter().enumerate() {
            if let Atom {
                symbol: Identifier::Function(0),
                parameters,
            } = premise
            {
                if parameters.len() >= 2 || (parameters.len() == 1 && !parameters[0].is_variable()) {
                    return Selection::Premise(i);
                }
            }
        }
        Selection::Conclusion
    }

    /// Resolution of r1 and r2
    /// r1 = |p| /\ q => r  (selected p)
    /// r2 = s /\ t => |c|  (selected c)
    /// if unfify(p, c) {
    ///     return asssigned(q /\ s /\ t => r, unify_context)
    /// }
    pub fn resolve(&self, other: &InnerRule) -> Option<InnerRule> {
        match (self.select(), other.select()) {
            (Selection::Premise(p), Selection::Conclusion) => {
                self.premises[p].unify(&other.conclusion).map(|bindings| {
                    let mut premises = self.premises.clone();
                    premises.remove(p);
                    premises.append(&mut other.premises.clone());

                    Rule {
                        conclusion: self.conclusion.clone(),
                        premises,
                    }
                    .apply(&bindings)
                })
            }
            (Selection::Conclusion, Selection::Premise(_)) => other.resolve(self),
            _ => None,
        }
    }
}
