use crate::ast::Rule;
use ptree::{Style, TreeItem};
use std::borrow::Cow;
use std::io::Write;

#[derive(Clone)]
pub struct DerivationTree {
    root: Rule<String>,
    subtrees: Vec<DerivationTree>,
}
impl DerivationTree {
    pub fn new(root: Rule<String>) -> DerivationTree {
        DerivationTree {
            root,
            subtrees: vec![],
        }
    }
    pub fn add_subtree(&mut self, subtree: DerivationTree) {
        self.subtrees.push(subtree)
    }
}
impl TreeItem for DerivationTree {
    type Child = Self;
    fn write_self<W: Write>(&self, f: &mut W, style: &Style) -> std::io::Result<()> {
        write!(f, "{}", style.paint(&self.root.to_string()))
    }
    fn children(&self) -> Cow<[Self::Child]> {
        Cow::from(self.subtrees.clone())
    }
}
