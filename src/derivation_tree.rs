use crate::ast::Rule;
use crate::resolution::Selection;
use ptree::{Color, Style, TreeItem};
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
        let mut selected_style = style.clone();
        selected_style.foreground = Some(Color::Red);
        selected_style.bold = true;

        let str_select = self.selection.as_ref().map(|selection| {
            let (Selection::Conclusion(a) | Selection::Premise(a, _)) = selection;
            a.to_string()
        });

        let string = self.root.to_string();
        if let Some(selected_str) = str_select {
            let mut non_selected = string.split(&selected_str);
            write!(
                f,
                "{}{}{}",
                style.paint(non_selected.next().unwrap_or(&String::new())),
                selected_style.paint(&selected_str),
                style.paint(non_selected.next().unwrap_or(&String::new()))
            )
        } else {
            write!(f, "{}", style.paint(string))
        }
    }
    fn children(&self) -> Cow<[Self::Child]> {
        Cow::from(self.subtrees.clone())
    }
}
