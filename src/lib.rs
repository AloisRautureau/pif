use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use logos_nom_bridge::Tokens;
use crate::ast::*;
use crate::identifiers::{Identifier, IdentifierServer};
use crate::parser::Parser;

mod ast;
mod lexer;
mod parser;
mod unify;
mod identifiers;

/// Sniffer's job is to saturate a set of rules, by deriving the current set until no
/// new rule can be added
#[derive(Default)]
pub struct Sniffer {
    generative_rules: HashSet<InnerRule>,
    axioms: HashSet<InnerAtom>,

    id_server: IdentifierServer,
}
impl Sniffer {
    /// Creates a Sniffer context from a `.pif` file
    pub fn new<P: AsRef<Path>>(file: P) -> Sniffer {
        // Parses the `.pif` file
        let mut file_contents = String::new();
        File::open(file).unwrap().read_to_string(&mut file_contents).unwrap();
        let parsed_rules = Parser::parse_rules(Tokens::new(&file_contents));

        // Then maps every string id to an inner identifier
        let mut sniffer = Sniffer::default();
        for rule in parsed_rules {
            if rule.premises.is_empty() {
                sniffer.axioms.insert(Atom::from((&rule.conclusion, &mut sniffer.id_server)));
            } else {
                sniffer.generative_rules.insert(Rule::from((&rule, &mut sniffer.id_server)));
            }
        }

        sniffer
    }

    /// Returns a derivation that results in a given rule if one exists
    pub fn find(&mut self, rule: &Rule<String>) -> Result<(), SaturationFailure> {
        /*
        let found = loop {
            if let Some(r) = self.rules.iter().find(|r| *r == rule) {
                break r;
            }
            self.saturate()?
        };
        Ok(())
         */
        todo!()
    }

    fn saturate(&mut self) -> Result<(), SaturationFailure> {
        // We derive new rules through resolution:
        // `p`, `p => q` |= `q`
        // In order to do so, we try to unify each and every axiom to every rule's premisses, until
        // one matches. When this happens, the conclusion can be added to the set of axioms
        // TODO: use a more clever selection function in order to avoid exponential growth

        let new_axioms = HashSet::new();

        for rule in &self.generative_rules {
            for premise in &rule.premises {
                // Try to unify the premise with any of the currently derived axioms
                // TODO: That's the entirety of the project tbh
            }
        }
        self.axioms = self.axioms.union(&new_axioms).cloned().collect();
        Ok(())
    }
}

/// Represents the result of a saturation attempt
pub enum SaturationFailure {
    Saturated,      // The saturation attempt did not create any new rule
    DerivedBottom,  // The saturation derived a contradiction
}