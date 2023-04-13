use crate::ast::{Atom, InnerAtom, InnerRule, InnerTerm, Rule, Term};
use crate::identifiers::Identifier;
use std::collections::HashMap;

struct VarInfo {
    pub marker: usize,
    pub bound: Option<InnerTerm>,
}

#[derive(Default)]
pub struct UnificationContext {
    marker: usize,
    nodes: HashMap<Identifier, VarInfo>,
}
impl UnificationContext {
    /// Returns `false` if the symbol is already bound
    pub fn bind(&mut self, symbol: Identifier, term: InnerTerm) -> bool {
        let node = self.nodes.entry(symbol).or_insert(VarInfo {
            marker: self.marker,
            bound: None
        });

        if node.bound.is_none() {
            node.bound = Some(term);
            true
        } else {
            false
        }
    }

    pub fn incr_marker(&mut self) {
        self.marker += 1
    }

    /// Returns `true` if the term was not already visited
    pub fn visit(&mut self, symbol: Identifier) -> bool {
        let node = self.nodes.entry(symbol).or_insert(VarInfo {
            marker: self.marker + 1,
            bound: None
        });

        if node.marker != self.marker {
            node.marker = self.marker;
            true
        } else {
            false
        }
    }

    pub fn deref<'a>(&'a self, mut symbol: &'a Identifier) -> Option<&InnerTerm> {
        let mut current_term = None;
        while let Some(VarInfo { bound: Some(t), .. }) = self.nodes.get(symbol) {
            current_term = Some(t);
            match t {
                Term::Variable { symbol: s } if s != symbol => symbol = s,
                _ => return current_term
            }
        }
        current_term
    }

    pub fn get_bindings(self) -> HashMap<InnerTerm, InnerTerm> {
        let mut map = HashMap::with_capacity(self.nodes.len());
        for symbol in self.nodes.keys().cloned() {
            let value = self.deref(&symbol).cloned();
            let ident = Term::Variable { symbol };
            map.insert(ident.clone(), value.unwrap_or(ident));
        }
        map
    }
}

impl InnerAtom {
    pub fn unify(&self, other: &InnerAtom) -> Option<HashMap<InnerTerm, InnerTerm>> {
        Term::from(self.clone()).unify(&Term::from(other.clone()))
    }
}

impl InnerTerm {
    /// Tries to unify this term with another
    pub fn unify(&self, other: &InnerTerm) -> Option<HashMap<InnerTerm, InnerTerm>> {
        let mut context = UnificationContext::default();
        let mut to_visit = vec![(self.clone(), other.clone())];

        while let Some((t, u)) = to_visit.pop() {
            // Finds leaves of terms `self` and `other`
            let (t, u) = (t.find(&context), u.find(&context));

            // If the leaves are equal, we can simply continue
            if t == u {
                continue
            }

            // Otherwise, our actions depend on the types of `leaf1` and `leaf2`
            match (&t, &u) {
                (Term::Variable { symbol }, Term::Variable { .. }) => {
                    context.bind(*symbol, u);
                }
                (x @ Term::Variable { symbol: x_id }, f @ Term::Function { .. })
                | (f @ Term::Function { .. }, x @ Term::Variable { symbol: x_id }) => {
                    if f.contains(x, &mut context) {
                        return None
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
                        context.bind(*f, u.clone());
                        for unify in f_params.clone().into_iter().zip(g_params.clone()) {
                            to_visit.push(unify)
                        }
                    } else {
                        return None
                    }
                }
            }
        }

        Some(context.get_bindings())
    }

    pub fn find(&self, context: &UnificationContext) -> InnerTerm {
        match self {
            Term::Variable { symbol } => if let Some(t) = context.deref(symbol) {
                t
            } else {
                self
            },
            t => t
        }.clone()
    }

    /// Checks if this term contains variable `t`
    pub fn contains(&self, u: &InnerTerm, context: &mut UnificationContext) -> bool {
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
                    return false
                }
            }
        }

        false
    }

    /// Applies a context to a term, setting its variables to their associated valued
    pub fn apply(&self, context: &UnificationContext) -> InnerTerm {
        match self {
            Term::Variable { .. } => {
                self.find(context).clone()
            }
            Term::Function { symbol, parameters } => Term::Function {
                symbol: *symbol,
                parameters: parameters.iter().map(|t| t.apply(context)).collect(),
            },
        }
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

        let mut context = UnificationContext::default();

        assert!(test_var_term.contains(&Term::Variable {
            symbol: Identifier::Variable(0)
        }, &mut context));
        assert!(!test_var_term.contains(&Term::Variable {
            symbol: Identifier::Variable(12)
        }, &mut context));

        assert!(test_fun_term.contains(&Term::Variable {
            symbol: Identifier::Variable(1)
        }, &mut context));
        assert!(test_fun_term.contains(&Term::Variable {
            symbol: Identifier::Variable(0)
        }, &mut context));
        assert!(test_fun_term.contains(&Term::Variable {
            symbol: Identifier::Variable(3)
        }, &mut context));
        assert!(!test_fun_term.contains(&Term::Variable {
            symbol: Identifier::Variable(12)
        }, &mut context));
    }

    #[test]
    fn apply_test() {
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

        let mut context = UnificationContext::default();
        context.bind(Identifier::Variable(0), Term::Function {
            symbol: Identifier::Function(2),
            parameters: vec![]
        });
        context.bind(Identifier::Variable(1), Term::Variable {
            symbol: Identifier::Variable(0),
        });
        context.bind(Identifier::Variable(2), Term::Function {
            symbol: Identifier::Function(4),
            parameters: vec![]
        });
        context.bind(Identifier::Variable(3), Term::Function {
            symbol: Identifier::Function(5),
            parameters: vec![]
        });

        let applied_var = test_var_term.apply(&context);
        let applied_fun = test_fun_term.apply(&context);
        assert_eq!(
            applied_var,
            Term::Function {
                symbol: Identifier::Function(2),
                parameters: vec![]
            }
        );
        assert_eq!(
            applied_fun,
            Term::Function {
                symbol: Identifier::Function(0),
                parameters: vec![
                    Term::Function {
                        symbol: Identifier::Function(2),
                        parameters: vec![]
                    },
                    Term::Function {
                        symbol: Identifier::Function(1),
                        parameters: vec![Term::Function {
                            symbol: Identifier::Function(2),
                            parameters: vec![]
                        }],
                    },
                    Term::Function {
                        symbol: Identifier::Function(5),
                        parameters: vec![]
                    },
                ]
            }
        )
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

        let mut context = UnificationContext::default();
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
            bindings.unwrap().get(&Term::Variable { symbol: Identifier::Variable(0) }),
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
            bindings.unwrap().get(&Term::Variable { symbol: Identifier::Variable(0) }),
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
            bindings.unwrap().get(&Term::Variable { symbol: Identifier::Variable(0) }),
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
            bindings.unwrap().get(&Term::Variable { symbol: Identifier::Variable(0) }),
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
            bindings.unwrap().get(&Term::Variable { symbol: Identifier::Variable(1) }),
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
