use std::borrow::Cow;
use std::io::Write;
use ptree::{Style, TreeItem};
use crate::ast::Atom;

#[derive(Clone)]
pub struct DerivationTree {
    root: Atom<String>,
    premises: Vec<DerivationTree>
}
impl DerivationTree {
    pub fn new(root: Atom<String>) -> DerivationTree {
        DerivationTree {
            root,
            premises: vec![]
        }
    }
    pub fn insert(&mut self, subtree: DerivationTree) {
        self.premises.push(subtree)
    }
}
impl TreeItem for DerivationTree {
    type Child = Self;
    fn write_self<W: Write>(&self, f: &mut W, style: &Style) -> std::io::Result<()> {
        write!(f, "{}", style.paint(&self.root.to_string()))
    }
    fn children(&self) -> Cow<[Self::Child]> {
        Cow::from(self.premises.clone())
    }
}