extern crate core;

use crate::ast::*;
use crate::derivation_tree::DerivationTree;
use crate::identifiers::{Identifier, IdentifierServer};
pub use crate::parser::Parser;
use itertools::Itertools;
use logos_nom_bridge::Tokens;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::resolution::Selection;

mod ast;
mod derivation_tree;
mod identifiers;
mod lexer;
mod parser;
mod resolution;
mod unify;
mod union_find;

/// Sniffer's job is to saturate a set of rules, by deriving the current set until no
/// new rule can be added
#[derive(Default)]
pub struct Sniffer {
    rules: HashSet<InnerRule>,
    derived_from: HashMap<InnerRule, (InnerRule, InnerRule)>,

    id_server: IdentifierServer,
}
impl Sniffer {
    /// Creates a Sniffer context from a `.pif` file
    pub fn new<P: AsRef<Path>>(file: P) -> Result<Sniffer, ()> {
        // Parses the `.pif` file
        let mut file_contents = String::new();
        if let Ok(mut file) = File::open(file) {
            file.read_to_string(&mut file_contents).unwrap();
        } else {
            return Err(());
        }
        let parsed_rules =
            Parser::parse_rules(Tokens::new(&file_contents)).expect("failed to parse file");

        // Then maps every string id to an inner identifier
        let mut sniffer = Sniffer::default();
        for rule in parsed_rules {
            let inner_rule = Rule::from((&rule, &mut sniffer.id_server));
            sniffer.add_rule(inner_rule, None);
        }
        Ok(sniffer)
    }

    /// Returns a derivation that results in a given rule if one exists
    pub fn find(&mut self, atom: &Atom<String>) -> Result<DerivationTree, SaturationFailure> {
        let inner_atom = Atom::from((atom, &mut self.id_server));
        let inner_rule = Rule {
            conclusion: inner_atom.clone(),
            premises: vec![],
        };

        // Create a selection function using the query
        let select = move |r: &InnerRule| {
            for p in r.premises.iter() {
                if p.symbol == inner_atom.symbol && p.parameters.iter().all(|p| !p.is_variable()) {
                    return Selection::Premise(p.clone())
                }
            }
            Selection::Conclusion(r.conclusion.clone())
        };

        // We keep saturating our rule set until we either find our atom or the set is fully saturated
        self.saturate(select);

        if self.rules.contains(&inner_rule) {
            Ok(self
                .derivation_tree(&Rule {
                    conclusion: atom.clone(),
                    premises: vec![],
                })
                .unwrap())
        } else {
            Err(SaturationFailure::Saturated)
        }
    }

    /// We derive new rules through resolution:
    /// A /\ B => C (B selected)
    /// D => B (B selected)
    /// then we have A /\ D => C
    ///
    /// Input : E
    /// Output : E*
    ///
    /// Pseudo code:
    /// E_1 = E
    /// E_2 = empty
    ///
    /// while E_1 != empty :
    ///     take C in E_1
    ///     add to E_1 every rule from the selected resolution between :
    ///         - C
    ///         - every element of E_2
    ///     add C to E_2
    /// return E_2
    /// 
    /// return None if it is finis hed because it means that we doesn't have find our solution
    /// return Some(DerivationTree ??) if it is finished because we have find our solution
    fn saturate(&mut self, select: impl Fn(&InnerRule) -> Selection) -> Option<DerivationTree> {
        let mut rules_set: Vec<_> = self.rules.clone().into_iter().collect();

        while let Some(rule) = rules_set.pop() {
            for other in &self.rules {
                if let Some(r) = rule.resolve(other, &select) {
                    if !self.rules.contains(&r) {
                        self.derived_from.insert(r.clone(), (rule.clone(), other.clone()));
                        rules_set.push(r)
                    }
                }
            }
            self.rules.insert(rule);
        }

        None
    }

    /// Adds a new rule, returning `false` if it was already present
    pub fn add_rule(
        &mut self,
        rule: InnerRule,
        derived_from: Option<(InnerRule, InnerRule)>,
    ) -> bool {
        if self.rules.insert(rule.clone()) {
            if let Some(derived_from) = derived_from {
                self.derived_from.insert(rule, derived_from);
            }
            true
        } else {
            false
        }
    }

    /// Returns the derivation tree for a given rule
    pub fn derivation_tree(&self, root: &Rule<String>) -> Option<DerivationTree> {
        fn inner(root: &InnerRule, sniffer: &Sniffer) -> Option<DerivationTree> {
            let mut decision_tree =
                DerivationTree::new(Rule::try_from((root, &sniffer.id_server)).ok()?);
            if let Some(premises) = sniffer.derived_from.get(root) {
                if let Some(tree) = inner(&premises.0, sniffer) {
                    decision_tree.add_subtree(tree)
                }
                if let Some(tree) = inner(&premises.1, sniffer) {
                    decision_tree.add_subtree(tree)
                }
            };
            Some(decision_tree)
        }

        let inner_rule = Rule::try_from((root, &self.id_server)).ok()?;

        let mut decision_tree = DerivationTree::new(root.clone());
        if let Some(premises) = self.derived_from.get(&inner_rule) {
            if let Some(tree) = inner(&premises.0, self) {
                decision_tree.add_subtree(tree)
            }
            if let Some(tree) = inner(&premises.1, self) {
                decision_tree.add_subtree(tree)
            }
        };
        Some(decision_tree)
    }

    pub fn rules_to_string(&self) -> String {
        self.rules
            .iter()
            .filter_map(|r| {
                Rule::try_from((r, &self.id_server))
                    .ok()
                    .map(|v| v.to_string())
            })
            .join("\n")
    }
}

/// Represents the result of a saturation attempt
pub enum SaturationFailure {
    Saturated,     // The saturation attempt did not create any new rule
    DerivedBottom, // The saturation derived a contradiction
}
