use crate::ast::{InnerAtom, InnerTerm, Term};
use crate::identifiers::Identifier;
use crate::union_find::UnionFind;
use rustc_hash::FxHashMap;
use std::collections::HashMap;

struct VarInfo {
    pub marker: usize,
    pub bound: Option<InnerTerm>,
}

#[derive(Default)]
pub struct UnificationGraph {
    marker: usize,
    nodes: FxHashMap<Identifier, VarInfo>,
    equivalence_classes: UnionFind<InnerTerm>,
}
impl UnificationGraph {
    /// Returns `false` if the symbol is already bound
    pub fn bind(&mut self, symbol: Identifier, term: InnerTerm) -> bool {
        let node = self.nodes.entry(symbol).or_insert(VarInfo {
            marker: self.marker,
            bound: None,
        });

        if node.bound.is_none() {
            node.bound = Some(term);
            true
        } else {
            false
        }
    }

    pub fn union(&mut self, x: InnerTerm, y: InnerTerm) {
        self.equivalence_classes.union(x, y);
    }

    pub fn incr_marker(&mut self) {
        self.marker += 1
    }

    /// Returns `true` if the term was not already visited
    pub fn visit(&mut self, symbol: Identifier) -> bool {
        let node = self.nodes.entry(symbol).or_insert(VarInfo {
            marker: self.marker + 1,
            bound: None,
        });

        if node.marker != self.marker {
            node.marker = self.marker;
            true
        } else {
            false
        }
    }

    pub fn deref_mut(&mut self, term: InnerTerm) -> InnerTerm {
        let root = self.equivalence_classes.find_equivalence_mut(term);
        if let Some(VarInfo { bound: Some(t), .. }) = self.nodes.get(root.symbol()) {
            self.equivalence_classes.find_equivalence_mut(t.clone())
        } else {
            root
        }
    }

    pub fn deref(&self, term: InnerTerm) -> Option<InnerTerm> {
        let root = self.equivalence_classes.find_equivalence(term)?;
        if let Some(VarInfo { bound: Some(t), .. }) = self.nodes.get(root.symbol()) {
            Some(
                self.equivalence_classes
                    .find_equivalence(t.clone())
                    .unwrap_or(t.clone()),
            )
        } else {
            Some(root)
        }
    }

    pub fn bindings(mut self) -> FxHashMap<InnerTerm, InnerTerm> {
        let mut bindings = HashMap::default();
        for t in self.equivalence_classes.clone().iter() {
            bindings.insert(t.clone(), self.deref_mut(t.clone()));
        }
        bindings
    }
}

impl InnerAtom {
    pub fn unify(&self, other: &InnerAtom) -> Option<FxHashMap<InnerTerm, InnerTerm>> {
        Term::from(self.clone()).unify(&Term::from(other.clone()))
    }
}

impl InnerTerm {
    /// Tries to unify this term with another
    pub fn unify(&self, other: &InnerTerm) -> Option<FxHashMap<InnerTerm, InnerTerm>> {
        let mut context = UnificationGraph::default();
        let mut to_visit = vec![(self.clone(), other.clone())];

        while let Some((t, u)) = to_visit.pop() {
            // Finds leaves of terms `self` and `other`
            let (t, u) = (context.deref_mut(t), context.deref_mut(u));

            // If the leaves are equal, we can simply continue
            if t == u {
                continue;
            }

            // Otherwise, our actions depend on the types of `leaf1` and `leaf2`
            match (&t, &u) {
                (Term::Variable { .. }, Term::Variable { .. }) => {
                    context.union(t, u);
                }
                (x @ Term::Variable { symbol: x_id }, f @ Term::Function { .. })
                | (f @ Term::Function { .. }, x @ Term::Variable { symbol: x_id }) => {
                    if f.contains(x, &mut context) {
                        return None;
                    } else {
                        context.bind(*x_id, f.clone());
                    }
                }
                (
                    Term::Function {
                        symbol: f,
                        parameters: f_params,
                    },
                    Term::Function {
                        symbol: g,
                        parameters: g_params,
                    },
                ) => {
                    if f == g && f_params.len() == g_params.len() {
                        context.union(t.clone(), u.clone());
                        for unify in f_params.clone().into_iter().zip(g_params.clone()) {
                            to_visit.push(unify)
                        }
                    } else {
                        return None;
                    }
                }
            }
        }

        Some(context.bindings())
    }

    /// Checks if this term contains variable `t`
    pub fn contains(&self, u: &InnerTerm, context: &mut UnificationGraph) -> bool {
        context.incr_marker();

        // Sadly Rust does not guarantee tail call optimizations (c.f https://dev.to/seanchen1991/the-story-of-tail-call-optimizations-in-rust-35hf)
        // Therefore, we must optimize this by hand
        let mut to_visit = vec![self];
        while let Some(t) = to_visit.pop() {
            if t == u {
                return true;
            }

            if let Term::Function { symbol, parameters } = t {
                if context.visit(*symbol) {
                    for param in parameters {
                        to_visit.push(param)
                    }
                } else {
                    return false;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_test() {
        let test_var_term = Term::Variable {
            symbol: Identifier::Variable(0),
        };
        let test_fun_term = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![
                Term::Variable {
                    symbol: Identifier::Variable(1),
                },
                Term::Function {
                    symbol: Identifier::Function(1),
                    parameters: vec![Term::Variable {
                        symbol: Identifier::Variable(0),
                    }],
                },
                Term::Variable {
                    symbol: Identifier::Variable(3),
                },
            ],
        };

        let mut context = UnificationGraph::default();

        assert!(test_var_term.contains(
            &Term::Variable {
                symbol: Identifier::Variable(0)
            },
            &mut context
        ));
        assert!(!test_var_term.contains(
            &Term::Variable {
                symbol: Identifier::Variable(12)
            },
            &mut context
        ));

        assert!(test_fun_term.contains(
            &Term::Variable {
                symbol: Identifier::Variable(1)
            },
            &mut context
        ));
        assert!(test_fun_term.contains(
            &Term::Variable {
                symbol: Identifier::Variable(0)
            },
            &mut context
        ));
        assert!(test_fun_term.contains(
            &Term::Variable {
                symbol: Identifier::Variable(3)
            },
            &mut context
        ));
        assert!(!test_fun_term.contains(
            &Term::Variable {
                symbol: Identifier::Variable(12)
            },
            &mut context
        ));
    }

    // The following tests on unification are issued from [wikipedia](https://en.wikipedia.org/wiki/Unification_(computer_science))
    #[test]
    fn unify_tautology_const_test() {
        let var = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![],
        };
        let var_copy = var.clone();

        assert!(var.unify(&var_copy).is_some());
    }

    #[test]
    fn unify_tautology_var_test() {
        let var = Term::Variable {
            symbol: Identifier::Variable(0),
        };
        let var_copy = var.clone();

        let mut _context = UnificationGraph::default();
        assert!(var.unify(&var_copy).is_some());
    }

    #[test]
    fn unify_diff_const_test() {
        let x = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![],
        };
        let y = Term::Function {
            symbol: Identifier::Function(1),
            parameters: vec![],
        };
        assert!(x.unify(&y).is_none())
    }

    #[test]
    fn unify_const_assign_test() {
        let var = Term::Variable {
            symbol: Identifier::Variable(0),
        };
        let cst = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![],
        };

        let bindings = var.unify(&cst);
        assert!(bindings.is_some());
        assert_eq!(
            bindings.unwrap().get(&Term::Variable {
                symbol: Identifier::Variable(0)
            }),
            Some(&Term::Function {
                symbol: Identifier::Function(0),
                parameters: vec![]
            })
        );
    }

    #[test]
    fn unify_aliasing_test() {
        let x = Term::Variable {
            symbol: Identifier::Variable(0),
        };
        let y = Term::Variable {
            symbol: Identifier::Variable(1),
        };

        let bindings = x.unify(&y);
        assert!(bindings.is_some());
        assert_eq!(
            bindings.unwrap().get(&Term::Variable {
                symbol: Identifier::Variable(0)
            }),
            Some(&Term::Variable {
                symbol: Identifier::Variable(1)
            })
        );
    }

    #[test]
    fn unify_function_test() {
        let complete_fun = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![
                Term::Function {
                    symbol: Identifier::Function(1),
                    parameters: vec![],
                },
                Term::Function {
                    symbol: Identifier::Function(2),
                    parameters: vec![],
                },
            ],
        };
        let incomplete_fun = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![
                Term::Function {
                    symbol: Identifier::Function(1),
                    parameters: vec![],
                },
                Term::Variable {
                    symbol: Identifier::Variable(0),
                },
            ],
        };

        let bindings = incomplete_fun.unify(&complete_fun);
        assert!(bindings.is_some());
        assert_eq!(
            bindings.unwrap().get(&Term::Variable {
                symbol: Identifier::Variable(0)
            }),
            Some(&Term::Function {
                symbol: Identifier::Function(2),
                parameters: vec![]
            })
        );
    }

    #[test]
    fn unify_diff_function_test() {
        let x = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![Term::Function {
                symbol: Identifier::Function(1),
                parameters: vec![],
            }],
        };
        let y = Term::Function {
            symbol: Identifier::Function(2),
            parameters: vec![Term::Function {
                symbol: Identifier::Function(1),
                parameters: vec![],
            }],
        };

        assert!(x.unify(&y).is_none());
    }

    #[test]
    fn unify_function_param_aliasing_test() {
        let x = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![Term::Variable {
                symbol: Identifier::Variable(0),
            }],
        };
        let y = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![Term::Variable {
                symbol: Identifier::Variable(1),
            }],
        };

        let bindings = x.unify(&y);
        assert!(bindings.is_some());
        assert_eq!(
            bindings.unwrap().get(&Term::Variable {
                symbol: Identifier::Variable(0)
            }),
            Some(&Term::Variable {
                symbol: Identifier::Variable(1)
            })
        );
    }

    #[test]
    fn unify_function_diff_arity_test() {
        let unary_fun = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![Term::Variable {
                symbol: Identifier::Variable(0),
            }],
        };
        let binary_fun = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![
                Term::Variable {
                    symbol: Identifier::Variable(1),
                },
                Term::Variable {
                    symbol: Identifier::Variable(2),
                },
            ],
        };

        assert!(unary_fun.unify(&binary_fun).is_none());
    }

    #[test]
    fn unify_inner_function_test() {
        let nested_fun = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![Term::Function {
                symbol: Identifier::Function(1),
                parameters: vec![Term::Variable {
                    symbol: Identifier::Variable(0),
                }],
            }],
        };
        let fun = Term::Function {
            symbol: Identifier::Function(0),
            parameters: vec![Term::Variable {
                symbol: Identifier::Variable(1),
            }],
        };

        let bindings = nested_fun.unify(&fun);
        assert!(bindings.is_some());
        assert_eq!(
            bindings.unwrap().get(&Term::Variable {
                symbol: Identifier::Variable(1)
            }),
            Some(&Term::Function {
                symbol: Identifier::Function(1),
                parameters: vec![Term::Variable {
                    symbol: Identifier::Variable(0)
                }]
            })
        );
    }

    // Hardcore optimization test, could burn your machine, handle with care
    #[ignore]
    #[test]
    fn the_revenge_of_bin_tree() {
        todo!()
    }
}
