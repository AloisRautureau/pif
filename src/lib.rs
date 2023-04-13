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

mod ast;
mod derivation_tree;
mod identifiers;
mod lexer;
mod parser;
mod unify;
mod union_find;

/// Sniffer's job is to saturate a set of rules, by deriving the current set until no
/// new rule can be added
#[derive(Default)]
pub struct Sniffer {
    rules: HashSet<InnerRule>,
    derived_from: HashMap<InnerRule, Vec<InnerRule>>,

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
            if rule.premises.is_empty() {
                let inner_axiom = Atom::from((&rule.conclusion, &mut sniffer.id_server));
                sniffer.add_axiom(inner_axiom);
            } else {
                sniffer
                    .clauses
                    .insert(Rule::from((&rule, &mut sniffer.id_server)));
            }
        }
        Ok(sniffer)
    }

    /// Returns a derivation that results in a given rule if one exists
    pub fn find(&mut self, atom: &Atom<String>) -> Result<DerivationTree, SaturationFailure> {
        let inner_atom = Atom::from((atom, &mut self.id_server));

        // We keep saturating our rule set until we either find our atom or the set is fully saturated
        while !self.axioms.contains(&inner_atom) {
            self.saturate()?
        }

        Ok(self.derivation_tree(atom).unwrap())
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
    ///     
    ///     add to E_1 every rule from the selected resolution between :
    ///         - C
    ///         - every element of E_2
    ///     
    ///     add C to E_2
    ///
    /// return E_2
    ///
    ///
    fn saturate(&mut self) -> Result<(), SaturationFailure> {
        let mut derived = vec![];
        for rule in &self.generative_rules {
            for input in self
                .axioms
                .iter()
                .cloned()
                .combinations(rule.premises.len())
            {
                if let Ok(resulting_rule) = rule.assign(input.as_slice()) {
                    derived.push(resulting_rule);
                }
            }
        }

        let mut modified = false;
        for r in derived {
            modified |= self.add_derived_axiom(r.conclusion, r.premises);
        }

        // Check if there are any new axioms that aren't already registered
        if modified {
            Ok(())
        } else {
            Err(SaturationFailure::Saturated)
        }
    }

    /// Adds a new derived axiom, return `false` if it was already present
    pub fn add_derived_axiom(&mut self, axiom: InnerAtom, derived_from: Vec<InnerAtom>) -> bool {
        self.derived_from
            .entry(axiom.clone())
            .or_insert_with(|| derived_from);
        self.axioms.insert(axiom)
    }

    /// Adds a new axiom, returning `false` if it was already present
    /*
    pub fn add_axiom(&mut self, axiom: InnerAtom) -> bool {
        self.derived_from.insert(axiom.clone(), vec![]);
        self.axioms.insert(axiom)
    }
    */

    pub fn add_inner_rule(&mut self, rule: InnerRule) -> bool {
        self.clauses.insert(rule);
        self.derived_from.insert(axiom.clone(), vec![]);
    }

    /// Returns the derivation tree for a given axiom
    pub fn derivation_tree(&self, root: &Atom<String>) -> Option<DerivationTree> {
        fn inner(root: &InnerAtom, sniffer: &Sniffer) -> Option<DerivationTree> {
            let mut decision_tree =
                DerivationTree::new(Atom::try_from((root, &sniffer.id_server)).ok()?);
            let premises = sniffer.derived_from.get(root)?;
            for pre in premises {
                decision_tree.insert(inner(pre, sniffer)?)
            }
            Some(decision_tree)
        }

        let inner_atom = Atom::try_from((root, &self.id_server)).ok()?;
        let premises = self.derived_from.get(&inner_atom)?;

        let mut decision_tree = DerivationTree::new(root.clone());
        for pre in premises {
            decision_tree.insert(inner(pre, self)?)
        }
        Some(decision_tree)
    }

    pub fn rules_to_string(&self) -> String {
        let mut rules = String::new();
        for rule in &self.generative_rules {
            let rule = Rule::try_from((rule, &self.id_server)).unwrap();
            rules.push_str(&format!("\t{}\n", rule));
        }
        rules
    }

    pub fn axioms_to_string(&self) -> String {
        let mut axioms = String::new();
        for axiom in &self.axioms {
            let axiom = Atom::try_from((axiom, &self.id_server)).unwrap();
            axioms.push_str(&format!("\t{}\n", axiom));
        }
        axioms
    }

    pub fn derived_from_to_string(&self) -> String {
        let mut derived_from = String::new();
        for (axiom, premises) in &self.derived_from {
            let axiom = Atom::try_from((axiom, &self.id_server)).unwrap();
            derived_from.push_str(&format!("\t{}:\n", axiom));

            // draw derivation tree

            for p in premises {
                let p = Atom::try_from((p, &self.id_server)).unwrap();
                // ptree::print_tree(self.derivation_tree(&p));
                derived_from.push_str(&format!("\t\t{}\n", p));
            }
        }
        derived_from
    }

    pub fn print_derived_from(&self) {
        for (axiom, _) in &self.derived_from {
            let atome = Atom::try_from((axiom, &self.id_server)).unwrap();
            ptree::print_tree(&self.derivation_tree(&atome).unwrap()).unwrap();
        }
    }
}

impl std::fmt::Display for Sniffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Rules:")?;
        write!(f, "{}", self.rules_to_string())?;

        writeln!(f, "Axioms: ")?;
        write!(f, "{}", self.axioms_to_string())?;

        writeln!(f, "Derived from: ")?;
        write!(f, "{}", self.derived_from_to_string())?;
        Ok(())
    }
}

/// Represents the result of a saturation attempt
pub enum SaturationFailure {
    Saturated,     // The saturation attempt did not create any new rule
    DerivedBottom, // The saturation derived a contradiction
}

pub enum Selection {
    Premise(usize),
    Conclusion,
}

/// Selection of an atom in a rule
/// (n in 0..+inf)
/// Input : r = A_1 /\ ... /\ A_n => B
/// Output : (Premise, i) or (Conclusion, None)
/// - (Premise, i) if A_i is selected
/// - (Conclusion, None) if B is selected
pub fn selection(r: InnerRule) -> Selection {
    Selection::Conclusion
}

/// Resolution of r1 and r2
/// r1 = |p| /\ q => r  (selected p)
/// r2 = s /\ t => |c|  (selected c)
/// if unfify(p, c) {
///     return asssigned(q /\ s /\ t => r, unify_context)
/// }
pub fn resolution(r1: InnerRule, r2: InnerRule) -> Option<InnerRule> {
    match (selection(r1), selection(r2)) {
        (Selection::Premise(p), Selection::Conclusion) => {
            if r1.premises[p] == r2.conclusion {
                // TODO
                // UNIFY r1.premises[p] and r2.conclusion
                let mut premises = r1.premises.clone().remove(p);

                premises.append(r2.premises.clone());

                Some(
                Rule {
                    conclusion: r1.conclusion.clone(),
                    premises,
                }
                )
            } else {
                None
            }
        }
        (TermType::Conclusion, TermType::Premise(_)) => resolution(r2, r1),
        _ => None,
    }
}
