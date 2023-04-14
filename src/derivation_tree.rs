use crate::ast::Rule;
use crate::resolution::Selection;
use ptree::{Style, TreeItem};
use std::borrow::Cow;
use std::io::Write;

#[derive(Clone)]
pub struct DerivationTree {
    root: Rule<String>,
    subtrees: Vec<DerivationTree>,
    selection: Option<Selection<String>>,
}
impl DerivationTree {
    pub fn new(root: Rule<String>) -> DerivationTree {
        DerivationTree {
            root,
            subtrees: vec![],
            selection: None,
        }
    }
    pub fn add_subtree(&mut self, subtree: DerivationTree) {
        self.subtrees.push(subtree)
    }
    pub fn set_selection(&mut self, selection: Selection<String>) {
        self.selection = Some(selection)
    }
}
impl TreeItem for DerivationTree {
    type Child = Self;
    fn write_self<W: Write>(&self, f: &mut W, style: &Style) -> std::io::Result<()> {
        if let Some(selection) = &self.selection {
            write!(
                f,
                "{}",
                style.paint(&self.root.selection_empathized_string(selection.clone()))
            )
        } else {
            write!(f, "{}", style.paint(&self.root.to_string()))
        }
    }
    fn children(&self) -> Cow<[Self::Child]> {
        Cow::from(self.subtrees.clone())
    }
}
