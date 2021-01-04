#[cfg(test)]
use std::collections::HashSet;

#[cfg(test)]
use crate::{
    core,
    evidence::{solver::solve_evar, Constraint, EVar},
    syntax::{self, Binop, Kind, Type},
    typecheck::{BoundVars, TypeError, Typechecker, UnifyKindContext, UnifyTypeContext},
    void::Void,
};

#[test]
fn infer_kind_test_1() {
    let mut tc = Typechecker::new();
    let expected = Ok((Type::Bool, Kind::Type));
    let actual = tc.infer_kind(&Type::Bool);
    assert_eq!(expected, actual)
}

#[test]
fn infer_kind_test_2() {
    let mut tc = Typechecker::new();
    let expected = Ok((Type::RowNil, Kind::Row));
    let actual = tc.infer_kind(&Type::RowNil);
    assert_eq!(expected, actual)
}

#[test]
fn infer_kind_test_3() {
    let mut tc = Typechecker::new();
    let expected = Err(TypeError::KindMismatch {
        pos: 0,
        context: UnifyKindContext {
            ty: Type::RowNil,
            has_kind: Kind::Type,
            unifying_types: None,
        },
        expected: Kind::Type,
        actual: Kind::Row,
    });
    let actual = tc.infer_kind(&Type::mk_rowcons(
        String::from("x"),
        Type::RowNil,
        Type::RowNil,
    ));
    assert_eq!(expected, actual)
}

#[test]
fn infer_kind_test_4() {
    let mut tc = Typechecker::new();
    let expected = Ok(Kind::Type);
    let actual = tc
        .infer_kind(&Type::mk_app(
            Type::Record,
            Type::mk_rowcons(String::from("x"), Type::Bool, Type::RowNil),
        ))
        .map(|(_, kind)| tc.zonk_kind(kind));
    assert_eq!(expected, actual)
}

#[test]
fn context_test_1() {
    let mut ctx = BoundVars::new();
    ctx.insert(&vec![
        (String::from("a"), Type::Unit::<usize>),
        (String::from("b"), Type::Bool),
        (String::from("c"), Type::String),
    ]);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![
                (String::from("a"), vec![2]),
                (String::from("b"), vec![1]),
                (String::from("c"), vec![0])
            ]
            .into_iter()
            .collect(),
            info: vec![
                (String::from("a"), Type::Unit),
                (String::from("b"), Type::Bool),
                (String::from("c"), Type::String),
            ]
        }
    );
    assert_eq!(ctx.lookup_name(&String::from("a")), Some((2, &Type::Unit)));
    assert_eq!(ctx.lookup_name(&String::from("b")), Some((1, &Type::Bool)));
    assert_eq!(
        ctx.lookup_name(&String::from("c")),
        Some((0, &Type::String))
    );
}

#[test]
#[should_panic]
fn context_test_2() {
    let mut ctx = BoundVars::new();
    ctx.insert(&vec![
        (String::from("a"), Type::Unit::<usize>),
        (String::from("a"), Type::Bool),
        (String::from("c"), Type::String),
    ]);
}

#[test]
fn context_test_3() {
    let mut ctx = BoundVars::new();
    ctx.insert(&vec![(String::from("a"), Type::Unit::<usize>)]);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![(String::from("a"), vec![0]),].into_iter().collect(),
            info: vec![(String::from("a"), Type::Unit),]
        }
    );
    ctx.insert(&vec![(String::from("b"), Type::Bool)]);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![(String::from("a"), vec![1]), (String::from("b"), vec![0]),]
                .into_iter()
                .collect(),
            info: vec![
                (String::from("a"), Type::Unit),
                (String::from("b"), Type::Bool),
            ]
        }
    );
    ctx.insert(&vec![(String::from("c"), Type::String)]);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![
                (String::from("a"), vec![2]),
                (String::from("b"), vec![1]),
                (String::from("c"), vec![0])
            ]
            .into_iter()
            .collect(),
            info: vec![
                (String::from("a"), Type::Unit),
                (String::from("b"), Type::Bool),
                (String::from("c"), Type::String),
            ]
        }
    );
}

#[test]
fn context_test_4() {
    let mut ctx = BoundVars::new();
    ctx.insert(&vec![(String::from("a"), Type::Unit::<usize>)]);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![(String::from("a"), vec![0]),].into_iter().collect(),
            info: vec![(String::from("a"), Type::Unit),]
        }
    );
    ctx.delete(1);
    assert_eq!(ctx, BoundVars::new())
}

#[test]
fn context_test_5() {
    let mut ctx = BoundVars::new();
    ctx.insert(&vec![(String::from("a"), Type::Unit::<usize>)]);
    ctx.insert(&vec![(String::from("b"), Type::Bool)]);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![(String::from("a"), vec![1]), (String::from("b"), vec![0])]
                .into_iter()
                .collect(),
            info: vec![
                (String::from("a"), Type::Unit),
                (String::from("b"), Type::Bool)
            ]
        }
    );
    ctx.delete(1);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![(String::from("a"), vec![0]),].into_iter().collect(),
            info: vec![(String::from("a"), Type::Unit),]
        }
    )
}

#[test]
fn context_test_6() {
    let mut ctx = BoundVars::new();
    ctx.insert(&vec![(String::from("a"), Type::Unit::<usize>)]);
    ctx.insert(&vec![(String::from("b"), Type::Bool)]);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![(String::from("a"), vec![1]), (String::from("b"), vec![0])]
                .into_iter()
                .collect(),
            info: vec![
                (String::from("a"), Type::Unit),
                (String::from("b"), Type::Bool)
            ]
        }
    );
    ctx.delete(2);
    assert_eq!(ctx, BoundVars::new())
}

#[test]
fn infer_pattern_test_1() {
    let mut tc = Typechecker::new();
    let pat = syntax::Pattern::Name(syntax::Spanned {
        pos: 0,
        item: String::from("x"),
    });
    assert_eq!(
        tc.infer_pattern(&pat),
        (
            core::Pattern::Name,
            syntax::Type::Meta(0),
            vec![(String::from("x"), syntax::Type::Meta(0))]
        )
    )
}

#[test]
fn infer_pattern_test_2() {
    let mut tc = Typechecker::new();
    let pat = syntax::Pattern::Record {
        names: vec![
            syntax::Spanned {
                pos: 0,
                item: String::from("x"),
            },
            syntax::Spanned {
                pos: 2,
                item: String::from("y"),
            },
            syntax::Spanned {
                pos: 4,
                item: String::from("z"),
            },
        ],
        rest: None,
    };
    assert_eq!(
        tc.infer_pattern(&pat),
        (
            core::Pattern::Record {
                names: vec![
                    core::Expr::mk_evar(2),
                    core::Expr::mk_evar(1),
                    core::Expr::mk_evar(0)
                ],
                rest: false
            },
            syntax::Type::mk_record(
                vec![
                    (String::from("x"), syntax::Type::Meta(0)),
                    (String::from("y"), syntax::Type::Meta(1)),
                    (String::from("z"), syntax::Type::Meta(2))
                ],
                None
            ),
            vec![
                (String::from("x"), syntax::Type::Meta(0)),
                (String::from("y"), syntax::Type::Meta(1)),
                (String::from("z"), syntax::Type::Meta(2)),
            ]
        )
    )
}

#[test]
fn infer_pattern_test_3() {
    let mut tc = Typechecker::new();
    let pat = syntax::Pattern::Record {
        names: vec![
            syntax::Spanned {
                pos: 0,
                item: String::from("x"),
            },
            syntax::Spanned {
                pos: 2,
                item: String::from("y"),
            },
            syntax::Spanned {
                pos: 4,
                item: String::from("z"),
            },
        ],
        rest: Some(syntax::Spanned {
            pos: 6,
            item: String::from("w"),
        }),
    };
    assert_eq!(
        tc.infer_pattern(&pat),
        (
            core::Pattern::Record {
                names: vec![
                    core::Expr::mk_evar(2),
                    core::Expr::mk_evar(1),
                    core::Expr::mk_evar(0)
                ],
                rest: true
            },
            syntax::Type::mk_record(
                vec![
                    (String::from("x"), syntax::Type::Meta(0)),
                    (String::from("y"), syntax::Type::Meta(1)),
                    (String::from("z"), syntax::Type::Meta(2))
                ],
                Some(syntax::Type::Meta(3))
            ),
            vec![
                (String::from("x"), syntax::Type::Meta(0)),
                (String::from("y"), syntax::Type::Meta(1)),
                (String::from("z"), syntax::Type::Meta(2)),
                (
                    String::from("w"),
                    syntax::Type::mk_record(Vec::new(), Some(syntax::Type::Meta(3)))
                ),
            ]
        )
    )
}

#[test]
fn infer_pattern_test_4() {
    let mut tc = Typechecker::new();
    let pat = syntax::Pattern::Variant {
        name: String::from("just"),
        arg: syntax::Spanned {
            pos: 5,
            item: String::from("x"),
        },
    };
    assert_eq!(
        tc.infer_pattern(&pat),
        (
            core::Pattern::Variant {
                name: String::from("just")
            },
            syntax::Type::mk_variant(
                vec![(String::from("just"), syntax::Type::Meta(0))],
                Some(syntax::Type::Meta(1))
            ),
            vec![(String::from("x"), syntax::Type::Meta(0))]
        )
    )
}

#[test]
fn infer_lam_test_1() {
    let mut tc = Typechecker::new();
    // \x -> x
    let term = syntax::Spanned {
        pos: 0,
        item: syntax::Expr::mk_lam(
            vec![syntax::Pattern::Name(syntax::Spanned {
                pos: 1,
                item: String::from("x"),
            })],
            syntax::Spanned {
                pos: 6,
                item: syntax::Expr::Var(String::from("x")),
            },
        ),
    };
    assert_eq!(
        tc.infer_expr(term),
        Ok((
            core::Expr::mk_lam(true, core::Expr::Var(0)),
            syntax::Type::mk_arrow(syntax::Type::Meta(0), syntax::Type::Meta(0))
        ))
    )
}

#[test]
fn infer_lam_test_2() {
    let mut tc = Typechecker::new();
    // \{x, y} -> x
    let term = syntax::Spanned {
        pos: 0,
        item: syntax::Expr::mk_lam(
            vec![syntax::Pattern::Record {
                names: vec![
                    syntax::Spanned {
                        pos: 2,
                        item: String::from("x"),
                    },
                    syntax::Spanned {
                        pos: 5,
                        item: String::from("y"),
                    },
                ],
                rest: None,
            }],
            syntax::Spanned {
                pos: 11,
                item: syntax::Expr::Var(String::from("x")),
            },
        ),
    };
    assert_eq!(
        tc.infer_expr(term),
        Ok((
            core::Expr::mk_lam(
                true,
                core::Expr::mk_case(
                    core::Expr::Var(0),
                    vec![core::Branch {
                        pattern: core::Pattern::Record {
                            names: vec![core::Expr::mk_evar(1), core::Expr::mk_evar(0)],
                            rest: false
                        },
                        body: core::Expr::Var(1)
                    }]
                )
            ),
            syntax::Type::mk_arrow(
                syntax::Type::mk_record(
                    vec![
                        (String::from("x"), syntax::Type::Meta(0)),
                        (String::from("y"), syntax::Type::Meta(1))
                    ],
                    None
                ),
                syntax::Type::Meta(0)
            )
        ))
    )
}

#[test]
fn infer_lam_test_3() {
    let mut tc = Typechecker::new();
    // \{x, y} -> y
    let term = syntax::Spanned {
        pos: 0,
        item: syntax::Expr::mk_lam(
            vec![syntax::Pattern::Record {
                names: vec![
                    syntax::Spanned {
                        pos: 2,
                        item: String::from("x"),
                    },
                    syntax::Spanned {
                        pos: 5,
                        item: String::from("y"),
                    },
                ],
                rest: None,
            }],
            syntax::Spanned {
                pos: 11,
                item: syntax::Expr::Var(String::from("y")),
            },
        ),
    };
    assert_eq!(
        tc.infer_expr(term),
        Ok((
            core::Expr::mk_lam(
                true,
                core::Expr::mk_case(
                    core::Expr::Var(0),
                    vec![core::Branch {
                        pattern: core::Pattern::Record {
                            names: vec![core::Expr::mk_evar(1), core::Expr::mk_evar(0)],
                            rest: false
                        },
                        body: core::Expr::Var(0)
                    }]
                )
            ),
            syntax::Type::mk_arrow(
                syntax::Type::mk_record(
                    vec![
                        (String::from("x"), syntax::Type::Meta(0)),
                        (String::from("y"), syntax::Type::Meta(1))
                    ],
                    None
                ),
                syntax::Type::Meta(1)
            )
        ))
    )
}

#[test]
fn infer_lam_test_4() {
    let mut tc = Typechecker::new();
    // \{x, y, ...z} -> z
    let term = syntax::Spanned {
        pos: 0,
        item: syntax::Expr::mk_lam(
            vec![syntax::Pattern::Record {
                names: vec![
                    syntax::Spanned {
                        pos: 2,
                        item: String::from("x"),
                    },
                    syntax::Spanned {
                        pos: 5,
                        item: String::from("y"),
                    },
                ],
                rest: Some(syntax::Spanned {
                    pos: 11,
                    item: String::from("z"),
                }),
            }],
            syntax::Spanned {
                pos: 17,
                item: syntax::Expr::Var(String::from("z")),
            },
        ),
    };
    let expected = Ok((
        core::Expr::mk_lam(
            true,
            core::Expr::mk_case(
                core::Expr::Var(0),
                vec![core::Branch {
                    pattern: core::Pattern::Record {
                        names: vec![core::Expr::mk_evar(1), core::Expr::mk_evar(0)],
                        rest: true,
                    },
                    body: core::Expr::Var(0),
                }],
            ),
        ),
        syntax::Type::mk_arrow(
            syntax::Type::mk_record(
                vec![
                    (String::from("x"), syntax::Type::Meta(0)),
                    (String::from("y"), syntax::Type::Meta(1)),
                ],
                Some(syntax::Type::Meta(2)),
            ),
            syntax::Type::mk_record(vec![], Some(syntax::Type::Meta(2))),
        ),
    ));
    let actual = tc.infer_expr(term);
    assert_eq!(expected, actual,)
}

#[test]
fn infer_lam_test_5() {
    let mut tc = Typechecker::new();
    // \f x -> f x
    let term = syntax::Spanned {
        pos: 0,
        item: syntax::Expr::mk_lam(
            vec![
                syntax::Pattern::Name(syntax::Spanned {
                    pos: 1,
                    item: String::from("f"),
                }),
                syntax::Pattern::Name(syntax::Spanned {
                    pos: 1,
                    item: String::from("x"),
                }),
            ],
            syntax::Expr::mk_app(
                syntax::Spanned {
                    pos: 8,
                    item: syntax::Expr::Var(String::from("f")),
                },
                syntax::Spanned {
                    pos: 10,
                    item: syntax::Expr::Var(String::from("x")),
                },
            ),
        ),
    };
    assert_eq!(
        tc.infer_expr(term)
            .map(|(expr, ty)| (expr, tc.zonk_type(ty))),
        Ok((
            core::Expr::mk_lam(
                true,
                core::Expr::mk_lam(
                    true,
                    core::Expr::mk_app(core::Expr::Var(1), core::Expr::Var(0))
                )
            ),
            syntax::Type::mk_arrow(
                syntax::Type::mk_arrow(syntax::Type::Meta(1), syntax::Type::Meta(3)),
                syntax::Type::mk_arrow(syntax::Type::Meta(1), syntax::Type::Meta(3))
            )
        ))
    )
}

#[test]
fn infer_array_test_1() {
    let mut tc = Typechecker::new();
    // [1, 2, 3]
    let term = syntax::Spanned {
        pos: 0,
        item: syntax::Expr::Array(vec![
            syntax::Spanned {
                pos: 1,
                item: syntax::Expr::Int(1),
            },
            syntax::Spanned {
                pos: 4,
                item: syntax::Expr::Int(2),
            },
            syntax::Spanned {
                pos: 7,
                item: syntax::Expr::Int(3),
            },
        ]),
    };
    assert_eq!(
        tc.infer_expr(term),
        Ok((
            core::Expr::Array(vec![
                core::Expr::Int(1),
                core::Expr::Int(2),
                core::Expr::Int(3)
            ]),
            syntax::Type::mk_app(syntax::Type::Array, syntax::Type::Int)
        ))
    )
}

#[test]
fn infer_array_test_2() {
    let mut tc = Typechecker::new();
    // [1, true, 3]
    let term = syntax::Spanned {
        pos: 0,
        item: syntax::Expr::Array(vec![
            syntax::Spanned {
                pos: 1,
                item: syntax::Expr::Int(1),
            },
            syntax::Spanned {
                pos: 4,
                item: syntax::Expr::True,
            },
            syntax::Spanned {
                pos: 10,
                item: syntax::Expr::Int(3),
            },
        ]),
    };
    assert_eq!(
        tc.infer_expr(term),
        Err(TypeError::TypeMismatch {
            pos: 4,
            context: UnifyTypeContext {
                expected: syntax::Type::Int,
                actual: syntax::Type::Bool,
            },
            expected: syntax::Type::Int,
            actual: syntax::Type::Bool
        })
    )
}

#[test]
fn unify_rows_test_1() {
    let mut tc = Typechecker::new();
    assert_eq!(
        tc.unify_type(
            &UnifyTypeContext {
                expected: syntax::Type::Unit,
                actual: syntax::Type::Unit
            },
            Type::mk_record(
                vec![
                    (String::from("x"), Type::Int),
                    (String::from("y"), Type::Bool)
                ],
                None
            ),
            Type::mk_record(
                vec![
                    (String::from("y"), Type::Bool),
                    (String::from("x"), Type::Int)
                ],
                None
            )
        ),
        Ok(())
    )
}

#[test]
fn unify_rows_test_2() {
    let mut tc = Typechecker::new();
    assert_eq!(
        tc.unify_type(
            &UnifyTypeContext {
                expected: syntax::Type::Unit,
                actual: syntax::Type::Unit,
            },
            Type::mk_record(
                vec![
                    (String::from("x"), Type::Int),
                    (String::from("x"), Type::Bool),
                    (String::from("y"), Type::Bool)
                ],
                None
            ),
            Type::mk_record(
                vec![
                    (String::from("y"), Type::Bool),
                    (String::from("x"), Type::Int),
                    (String::from("x"), Type::Bool)
                ],
                None
            )
        ),
        Ok(())
    )
}

#[test]
fn unify_rows_test_3() {
    let mut tc = Typechecker::new();
    assert_eq!(
        tc.unify_type(
            &UnifyTypeContext {
                expected: syntax::Type::Unit,
                actual: syntax::Type::Unit,
            },
            Type::mk_record(
                vec![
                    (String::from("x"), Type::Int),
                    (String::from("x"), Type::Bool),
                    (String::from("y"), Type::Bool)
                ],
                None
            ),
            Type::mk_record(
                vec![
                    (String::from("x"), Type::Int),
                    (String::from("y"), Type::Bool),
                    (String::from("x"), Type::Bool)
                ],
                None
            )
        ),
        Ok(())
    )
}

#[test]
fn unify_rows_test_4() {
    let mut tc = Typechecker::new();
    assert_eq!(
        tc.unify_type(
            &UnifyTypeContext {
                expected: syntax::Type::Unit,
                actual: syntax::Type::Unit
            },
            Type::mk_record(
                vec![
                    (String::from("x"), Type::Int),
                    (String::from("x"), Type::Bool),
                    (String::from("y"), Type::Bool)
                ],
                None
            ),
            Type::mk_record(
                vec![
                    (String::from("x"), Type::Int),
                    (String::from("y"), Type::Bool),
                    (String::from("x"), Type::Int)
                ],
                None
            )
        ),
        Err(TypeError::TypeMismatch {
            pos: 0,
            context: UnifyTypeContext {
                expected: syntax::Type::Unit,
                actual: syntax::Type::Unit
            },
            expected: Type::Bool,
            actual: Type::Int
        })
    )
}

#[test]
fn infer_record_test_1() {
    let mut tc = Typechecker::new();
    // {}
    let term = syntax::Expr::mk_record(Vec::new(), None);
    assert_eq!(
        tc.infer_expr(syntax::Spanned { pos: 0, item: term })
            .map(|(expr, ty)| (expr, tc.zonk_type(ty))),
        Ok((
            core::Expr::mk_record(Vec::new(), None),
            syntax::Type::mk_record(Vec::new(), None)
        ))
    )
}

#[test]
fn infer_record_test_2() {
    let mut tc = Typechecker::new();
    // { x = 1, y = true }
    let term = syntax::Expr::mk_record(
        vec![
            (
                String::from("x"),
                syntax::Spanned {
                    pos: 2,
                    item: syntax::Expr::Int(1),
                },
            ),
            (
                String::from("y"),
                syntax::Spanned {
                    pos: 13,
                    item: syntax::Expr::True,
                },
            ),
        ],
        None,
    );
    assert_eq!(
        tc.infer_expr(syntax::Spanned { pos: 0, item: term })
            .map(|(expr, ty)| (expr, tc.zonk_type(ty))),
        Ok((
            core::Expr::mk_record(
                vec![
                    (core::Expr::mk_evar(1), core::Expr::Int(1)),
                    (core::Expr::mk_evar(0), core::Expr::True)
                ],
                None
            ),
            syntax::Type::mk_record(
                vec![
                    (String::from("x"), syntax::Type::Int),
                    (String::from("y"), syntax::Type::Bool)
                ],
                None
            )
        ))
    )
}

#[test]
fn infer_record_test_3() {
    let mut tc = Typechecker::new();
    // { x = 1, y = true, ...{ z = 'c' } }
    let term = syntax::Expr::mk_record(
        vec![
            (
                String::from("x"),
                syntax::Spanned {
                    pos: 2,
                    item: syntax::Expr::Int(1),
                },
            ),
            (
                String::from("y"),
                syntax::Spanned {
                    pos: 13,
                    item: syntax::Expr::True,
                },
            ),
        ],
        Some(syntax::Spanned {
            pos: 22,
            item: syntax::Expr::mk_record(
                vec![(
                    String::from("z"),
                    syntax::Spanned {
                        pos: 24,
                        item: syntax::Expr::Char('c'),
                    },
                )],
                None,
            ),
        }),
    );
    assert_eq!(
        tc.infer_expr(syntax::Spanned { pos: 0, item: term })
            .map(|(expr, ty)| (expr, tc.zonk_type(ty))),
        Ok((
            core::Expr::mk_record(
                vec![
                    (core::Expr::mk_evar(1), core::Expr::Int(1)),
                    (core::Expr::mk_evar(0), core::Expr::True)
                ],
                Some(core::Expr::mk_record(
                    vec![(core::Expr::mk_evar(2), core::Expr::Char('c'))],
                    None
                ))
            ),
            syntax::Type::mk_record(
                vec![
                    (String::from("x"), syntax::Type::Int),
                    (String::from("y"), syntax::Type::Bool),
                    (String::from("z"), syntax::Type::Char)
                ],
                None
            )
        ))
    )
}

#[test]
fn infer_record_test_4() {
    let mut tc = Typechecker::new();
    // { x = 1, y = true, ...1 }
    let term = syntax::Expr::mk_record(
        vec![
            (
                String::from("x"),
                syntax::Spanned {
                    pos: 2,
                    item: syntax::Expr::Int(1),
                },
            ),
            (
                String::from("y"),
                syntax::Spanned {
                    pos: 13,
                    item: syntax::Expr::True,
                },
            ),
        ],
        Some(syntax::Spanned {
            pos: 22,
            item: syntax::Expr::Int(1),
        }),
    );
    assert_eq!(
        tc.infer_expr(syntax::Spanned { pos: 0, item: term })
            .map(|(expr, ty)| (expr, tc.zonk_type(ty))),
        Err(TypeError::TypeMismatch {
            pos: 22,
            context: UnifyTypeContext {
                expected: syntax::Type::mk_record(Vec::new(), Some(syntax::Type::Meta(0))),
                actual: syntax::Type::Int
            },
            expected: syntax::Type::mk_record(Vec::new(), Some(syntax::Type::Meta(0))),
            actual: syntax::Type::Int
        })
    )
}

#[test]
fn infer_case_1() {
    let mut tc = Typechecker::new();
    /*
    \x -> case x of
      X a -> a
    */
    let term = syntax::Spanned {
        pos: 0,
        item: syntax::Expr::mk_lam(
            vec![syntax::Pattern::Name(syntax::Spanned {
                pos: 1,
                item: String::from("x"),
            })],
            syntax::Spanned {
                pos: 6,
                item: syntax::Expr::mk_case(
                    syntax::Spanned {
                        pos: 11,
                        item: syntax::Expr::Var(String::from("x")),
                    },
                    vec![syntax::Branch {
                        pattern: syntax::Spanned {
                            pos: 18,
                            item: syntax::Pattern::Variant {
                                name: String::from("X"),
                                arg: syntax::Spanned {
                                    pos: 20,
                                    item: String::from("a"),
                                },
                            },
                        },
                        body: syntax::Spanned {
                            pos: 25,
                            item: syntax::Expr::Var(String::from("a")),
                        },
                    }],
                ),
            },
        ),
    };
    assert_eq!(
        tc.infer_expr(term)
            .map(|(expr, ty)| (expr, tc.zonk_type(ty))),
        Ok((
            core::Expr::mk_lam(
                true,
                core::Expr::mk_case(
                    core::Expr::Var(0),
                    vec![core::Branch {
                        pattern: core::Pattern::Variant {
                            name: String::from("X")
                        },
                        body: core::Expr::Var(0)
                    }]
                )
            ),
            syntax::Type::mk_arrow(
                syntax::Type::mk_variant(vec![(String::from("X"), syntax::Type::Meta(3))], None),
                syntax::Type::Meta(3)
            )
        ))
    )
}

#[test]
fn infer_case_2() {
    let mut tc = Typechecker::new();
    /*
    \x -> case x of
      Left a -> a
      Right b -> b
    */
    let term = syntax::Spanned {
        pos: 0,
        item: syntax::Expr::mk_lam(
            vec![syntax::Pattern::Name(syntax::Spanned {
                pos: 1,
                item: String::from("x"),
            })],
            syntax::Spanned {
                pos: 6,
                item: syntax::Expr::mk_case(
                    syntax::Spanned {
                        pos: 11,
                        item: syntax::Expr::Var(String::from("x")),
                    },
                    vec![
                        syntax::Branch {
                            pattern: syntax::Spanned {
                                pos: 18,
                                item: syntax::Pattern::Variant {
                                    name: String::from("Left"),
                                    arg: syntax::Spanned {
                                        pos: 23,
                                        item: String::from("a"),
                                    },
                                },
                            },
                            body: syntax::Spanned {
                                pos: 28,
                                item: syntax::Expr::Var(String::from("a")),
                            },
                        },
                        syntax::Branch {
                            pattern: syntax::Spanned {
                                pos: 32,
                                item: syntax::Pattern::Variant {
                                    name: String::from("Right"),
                                    arg: syntax::Spanned {
                                        pos: 34,
                                        item: String::from("b"),
                                    },
                                },
                            },
                            body: syntax::Spanned {
                                pos: 39,
                                item: syntax::Expr::Var(String::from("b")),
                            },
                        },
                    ],
                ),
            },
        ),
    };
    assert_eq!(
        tc.infer_expr(term)
            .map(|(expr, ty)| (expr, tc.zonk_type(ty))),
        Ok((
            core::Expr::mk_lam(
                true,
                core::Expr::mk_case(
                    core::Expr::Var(0),
                    vec![
                        core::Branch {
                            pattern: core::Pattern::Variant {
                                name: String::from("Left")
                            },
                            body: core::Expr::Var(0)
                        },
                        core::Branch {
                            pattern: core::Pattern::Variant {
                                name: String::from("Right")
                            },
                            body: core::Expr::Var(0)
                        },
                    ]
                )
            ),
            syntax::Type::mk_arrow(
                syntax::Type::mk_variant(
                    vec![
                        (String::from("Left"), syntax::Type::Meta(5)),
                        (String::from("Right"), syntax::Type::Meta(5))
                    ],
                    None
                ),
                syntax::Type::Meta(5)
            )
        ))
    )
}

#[test]
fn infer_case_3() {
    let mut tc = Typechecker::new();
    /*
    \x -> case x of
      Left a -> a
      Right b -> b
      _ -> 1
    */
    let term = syntax::Spanned {
        pos: 0,
        item: syntax::Expr::mk_lam(
            vec![syntax::Pattern::Name(syntax::Spanned {
                pos: 1,
                item: String::from("x"),
            })],
            syntax::Spanned {
                pos: 6,
                item: syntax::Expr::mk_case(
                    syntax::Spanned {
                        pos: 11,
                        item: syntax::Expr::Var(String::from("x")),
                    },
                    vec![
                        syntax::Branch {
                            pattern: syntax::Spanned {
                                pos: 18,
                                item: syntax::Pattern::Variant {
                                    name: String::from("Left"),
                                    arg: syntax::Spanned {
                                        pos: 23,
                                        item: String::from("a"),
                                    },
                                },
                            },
                            body: syntax::Spanned {
                                pos: 28,
                                item: syntax::Expr::Var(String::from("a")),
                            },
                        },
                        syntax::Branch {
                            pattern: syntax::Spanned {
                                pos: 32,
                                item: syntax::Pattern::Variant {
                                    name: String::from("Right"),
                                    arg: syntax::Spanned {
                                        pos: 34,
                                        item: String::from("b"),
                                    },
                                },
                            },
                            body: syntax::Spanned {
                                pos: 39,
                                item: syntax::Expr::Var(String::from("b")),
                            },
                        },
                        syntax::Branch {
                            pattern: syntax::Spanned {
                                pos: 43,
                                item: syntax::Pattern::Wildcard,
                            },
                            body: syntax::Spanned {
                                pos: 48,
                                item: syntax::Expr::Int(1),
                            },
                        },
                    ],
                ),
            },
        ),
    };
    assert_eq!(
        tc.infer_expr(term)
            .map(|(expr, ty)| (expr, tc.zonk_type(ty))),
        Ok((
            core::Expr::mk_lam(
                true,
                core::Expr::mk_case(
                    core::Expr::Var(0),
                    vec![
                        core::Branch {
                            pattern: core::Pattern::Variant {
                                name: String::from("Left")
                            },
                            body: core::Expr::Var(0)
                        },
                        core::Branch {
                            pattern: core::Pattern::Variant {
                                name: String::from("Right")
                            },
                            body: core::Expr::Var(0)
                        },
                        core::Branch {
                            pattern: core::Pattern::Wildcard,
                            body: core::Expr::Int(1)
                        },
                    ]
                )
            ),
            syntax::Type::mk_arrow(
                syntax::Type::mk_variant(
                    vec![
                        (String::from("Left"), syntax::Type::Int),
                        (String::from("Right"), syntax::Type::Int)
                    ],
                    Some(syntax::Type::Meta(7))
                ),
                syntax::Type::Int
            )
        ))
    )
}

#[test]
fn infer_case_4() {
    let mut tc = Typechecker::new();
    /*
    \x -> case x of
      Left a -> a
      Left b -> b
      _ -> 1
    */
    let term = syntax::Spanned {
        pos: 0,
        item: syntax::Expr::mk_lam(
            vec![syntax::Pattern::Name(syntax::Spanned {
                pos: 1,
                item: String::from("x"),
            })],
            syntax::Spanned {
                pos: 6,
                item: syntax::Expr::mk_case(
                    syntax::Spanned {
                        pos: 11,
                        item: syntax::Expr::Var(String::from("x")),
                    },
                    vec![
                        syntax::Branch {
                            pattern: syntax::Spanned {
                                pos: 18,
                                item: syntax::Pattern::Variant {
                                    name: String::from("Left"),
                                    arg: syntax::Spanned {
                                        pos: 23,
                                        item: String::from("a"),
                                    },
                                },
                            },
                            body: syntax::Spanned {
                                pos: 28,
                                item: syntax::Expr::Var(String::from("a")),
                            },
                        },
                        syntax::Branch {
                            pattern: syntax::Spanned {
                                pos: 32,
                                item: syntax::Pattern::Variant {
                                    name: String::from("Left"),
                                    arg: syntax::Spanned {
                                        pos: 34,
                                        item: String::from("b"),
                                    },
                                },
                            },
                            body: syntax::Spanned {
                                pos: 38,
                                item: syntax::Expr::Var(String::from("b")),
                            },
                        },
                        syntax::Branch {
                            pattern: syntax::Spanned {
                                pos: 42,
                                item: syntax::Pattern::Wildcard,
                            },
                            body: syntax::Spanned {
                                pos: 47,
                                item: syntax::Expr::Int(1),
                            },
                        },
                    ],
                ),
            },
        ),
    };
    assert_eq!(
        tc.infer_expr(term)
            .map(|(expr, ty)| (expr, tc.zonk_type(ty))),
        Err(TypeError::RedundantPattern { pos: 32 })
    )
}

#[test]
fn infer_record_1() {
    let mut tc = Typechecker::new();
    let expected_expr = core::Expr::mk_record(
        vec![
            (core::Expr::EVar(EVar(2)), core::Expr::False),
            (core::Expr::EVar(EVar(1)), core::Expr::String(Vec::new())),
            (core::Expr::EVar(EVar(0)), core::Expr::Int(0)),
        ],
        None,
    );
    let expected_ty = Type::mk_record(
        vec![
            (String::from("z"), Type::Bool),
            (String::from("y"), Type::String),
            (String::from("x"), Type::Int),
        ],
        None,
    );
    let expected_result = Ok((expected_expr, expected_ty));
    let expr = syntax::Spanned {
        pos: 0,
        item: syntax::Expr::mk_record(
            vec![
                (
                    String::from("z"),
                    syntax::Spanned {
                        pos: 1,
                        item: syntax::Expr::False,
                    },
                ),
                (
                    String::from("y"),
                    syntax::Spanned {
                        pos: 2,
                        item: syntax::Expr::String(Vec::new()),
                    },
                ),
                (
                    String::from("x"),
                    syntax::Spanned {
                        pos: 3,
                        item: syntax::Expr::Int(0),
                    },
                ),
            ],
            None,
        ),
    };
    let actual_result = tc
        .infer_expr(expr)
        .map(|(expr, ty)| (expr, tc.zonk_type(ty)));
    assert_eq!(expected_result, actual_result, "checking results");

    let (actual_expr, _actual_ty) = actual_result.unwrap();

    let mut evars: HashSet<EVar> = HashSet::new();
    let _: Result<core::Expr, Void> = actual_expr.subst_evar(&mut |ev| {
        evars.insert(*ev);
        Ok(core::Expr::EVar(*ev))
    });

    assert_eq!(3, evars.len(), "checking number of EVars");

    let ev0 = evars.get(&EVar(0)).unwrap();
    let ev1 = evars.get(&EVar(1)).unwrap();
    let ev2 = evars.get(&EVar(2)).unwrap();

    assert_eq!(
        Ok((
            core::Expr::Int(0),
            Constraint::HasField {
                field: String::from("x"),
                rest: Type::RowNil
            }
        )),
        solve_evar(&mut tc, *ev0).map(|(expr, constraint)| (expr, tc.zonk_constraint(constraint)))
    );

    assert_eq!(
        Ok((
            core::Expr::mk_binop(Binop::Add, core::Expr::Int(1), core::Expr::Int(0)),
            Constraint::HasField {
                field: String::from("y"),
                rest: Type::mk_rows(vec![(String::from("x"), Type::Int)], None)
            }
        )),
        solve_evar(&mut tc, *ev1).map(|(expr, constraint)| (expr, tc.zonk_constraint(constraint)))
    );

    assert_eq!(
        Ok((
            core::Expr::mk_binop(
                Binop::Add,
                core::Expr::Int(1),
                core::Expr::mk_binop(Binop::Add, core::Expr::Int(1), core::Expr::Int(0))
            ),
            Constraint::HasField {
                field: String::from("z"),
                rest: Type::mk_rows(
                    vec![
                        (String::from("y"), Type::String),
                        (String::from("x"), Type::Int)
                    ],
                    None
                )
            }
        )),
        solve_evar(&mut tc, *ev2).map(|(expr, constraint)| (expr, tc.zonk_constraint(constraint)))
    );
}

#[test]
fn check_definition_1() {
    let mut tc = Typechecker::new();
    /*
    id : a -> a
    id x = x
    */
    let decl = syntax::Spanned {
        pos: 0,
        item: syntax::Declaration::Definition {
            name: String::from("id"),
            ty: syntax::Type::mk_arrow(
                syntax::Type::Var(String::from("a")),
                syntax::Type::Var(String::from("a")),
            ),
            args: vec![syntax::Pattern::Name(syntax::Spanned {
                pos: 14,
                item: String::from("x"),
            })],
            body: syntax::Spanned {
                pos: 18,
                item: syntax::Expr::Var(String::from("x")),
            },
        },
    };
    assert_eq!(
        tc.check_declaration(decl),
        Ok(core::Declaration::Definition {
            name: String::from("id"),
            sig: core::TypeSig {
                ty_vars: vec![syntax::Kind::Type],
                body: syntax::Type::mk_arrow(syntax::Type::Var(0), syntax::Type::Var(0))
            },
            body: core::Expr::mk_lam(true, core::Expr::Var(0))
        })
    )
}

#[test]
fn check_definition_2() {
    let mut tc = Typechecker::new();
    /*
    thing : { r } -> { x : Int, r }
    thing r = { x = 0, ..r }
    */
    let decl = syntax::Spanned {
        pos: 0,
        item: syntax::Declaration::Definition {
            name: String::from("thing"),
            ty: syntax::Type::mk_arrow(
                syntax::Type::mk_record(Vec::new(), Some(Type::Var(String::from("r")))),
                syntax::Type::mk_record(
                    vec![(String::from("x"), Type::Int)],
                    Some(syntax::Type::Var(String::from("r"))),
                ),
            ),
            args: vec![syntax::Pattern::Name(syntax::Spanned {
                pos: 37,
                item: String::from("r"),
            })],
            body: syntax::Spanned {
                pos: 41,
                item: syntax::Expr::mk_record(
                    vec![(
                        String::from("x"),
                        syntax::Spanned {
                            pos: 47,
                            item: syntax::Expr::Int(0),
                        },
                    )],
                    Some(syntax::Spanned {
                        pos: 52,
                        item: syntax::Expr::Var(String::from("r")),
                    }),
                ),
            },
        },
    };
    let expected = Ok(core::Declaration::Definition {
        name: String::from("thing"),
        sig: core::TypeSig {
            ty_vars: vec![syntax::Kind::Row],
            body: syntax::Type::mk_fatarrow(
                syntax::Type::mk_hasfield(String::from("x"), Type::Var(0)),
                syntax::Type::mk_arrow(
                    syntax::Type::mk_record(Vec::new(), Some(Type::Var(0))),
                    syntax::Type::mk_record(
                        vec![(String::from("x"), Type::Int)],
                        Some(syntax::Type::Var(0)),
                    ),
                ),
            ),
        },
        body: core::Expr::mk_lam(
            true,
            core::Expr::mk_lam(
                true,
                core::Expr::mk_extend(core::Expr::Var(1), core::Expr::Int(0), core::Expr::Var(0)),
            ),
        ),
    });
    let actual = tc.check_declaration(decl);
    assert_eq!(expected, actual)
}

#[test]
fn check_definition_3() {
    let mut tc = Typechecker::new();
    /*
    thing : { z : Bool, y : String, x : Int }
    thing = { z = False, y = "", x = 0 }
    */
    let decl = syntax::Spanned {
        pos: 0,
        item: syntax::Declaration::Definition {
            name: String::from("thing"),
            ty: syntax::Type::mk_record(
                vec![
                    (String::from("z"), Type::Bool),
                    (String::from("y"), Type::String),
                    (String::from("x"), Type::Int),
                ],
                None,
            ),
            args: Vec::new(),
            body: syntax::Spanned {
                pos: 1,
                item: syntax::Expr::mk_record(
                    vec![
                        (
                            String::from("z"),
                            syntax::Spanned {
                                pos: 3,
                                item: syntax::Expr::False,
                            },
                        ),
                        (
                            String::from("y"),
                            syntax::Spanned {
                                pos: 4,
                                item: syntax::Expr::String(Vec::new()),
                            },
                        ),
                        (
                            String::from("x"),
                            syntax::Spanned {
                                pos: 5,
                                item: syntax::Expr::Int(0),
                            },
                        ),
                    ],
                    None,
                ),
            },
        },
    };
    let expected = Ok(core::Declaration::Definition {
        name: String::from("thing"),
        sig: core::TypeSig {
            ty_vars: Vec::new(),
            body: syntax::Type::mk_record(
                vec![
                    (String::from("z"), Type::Bool),
                    (String::from("y"), Type::String),
                    (String::from("x"), Type::Int),
                ],
                None,
            ),
        },
        body: core::Expr::mk_record(
            vec![
                (core::Expr::Int(2), core::Expr::False),
                (core::Expr::Int(1), core::Expr::String(Vec::new())),
                (core::Expr::Int(0), core::Expr::Int(0)),
            ],
            None,
        ),
    });
    let actual = tc.check_declaration(decl);
    assert_eq!(expected, actual)
}

#[test]
fn check_definition_4() {
    let mut tc = Typechecker::new();
    /*
    getx : { x : Int, r } -> Int
    getx { x, r } = x
    */
    let decl = syntax::Spanned {
        pos: 0,
        item: syntax::Declaration::Definition {
            name: String::from("getx"),
            ty: syntax::Type::mk_arrow(
                syntax::Type::mk_record(
                    vec![(String::from("x"), Type::Int)],
                    Some(syntax::Type::Var(String::from("r"))),
                ),
                syntax::Type::Int,
            ),
            args: vec![syntax::Pattern::Record {
                names: vec![syntax::Spanned {
                    pos: 1,
                    item: String::from("x"),
                }],
                rest: Some(syntax::Spanned {
                    pos: 2,
                    item: String::from("r"),
                }),
            }],
            body: syntax::Spanned {
                pos: 2,
                item: syntax::Expr::Var(String::from("x")),
            },
        },
    };
    assert_eq!(
        tc.check_declaration(decl),
        Ok(core::Declaration::Definition {
            name: String::from("getx"),
            sig: core::TypeSig {
                ty_vars: vec![syntax::Kind::Row],
                body: syntax::Type::mk_fatarrow(
                    syntax::Type::mk_hasfield(String::from("x"), syntax::Type::Var(0)),
                    syntax::Type::mk_arrow(
                        syntax::Type::mk_record(
                            vec![(String::from("x"), syntax::Type::Int)],
                            Some(syntax::Type::Var(0))
                        ),
                        syntax::Type::Int
                    ),
                )
            },
            body: core::Expr::mk_lam(
                true,
                core::Expr::mk_lam(
                    true,
                    core::Expr::mk_case(
                        core::Expr::Var(0),
                        vec![core::Branch {
                            pattern: core::Pattern::Record {
                                names: vec![core::Expr::Var(1)],
                                rest: true
                            },
                            body: core::Expr::Var(1)
                        }]
                    )
                )
            )
        })
    )
}

#[test]
fn kind_occurs_1() {
    let mut tc = Typechecker::new();
    let v1 = tc.fresh_kindvar();
    let v2 = tc.fresh_kindvar();
    assert_eq!(
        tc.unify_kind(
            &UnifyKindContext {
                ty: Type::Unit,
                has_kind: Kind::Type,
                unifying_types: None
            },
            v1.clone(),
            Kind::mk_arrow(v1.clone(), v2.clone())
        ),
        Err(TypeError::KindOccurs {
            pos: 0,
            meta: 0,
            kind: Kind::mk_arrow(v1, v2)
        })
    )
}

#[test]
fn type_occurs_1() {
    let mut tc = Typechecker::new();
    let v1 = tc.fresh_typevar(Kind::Type);
    let v2 = tc.fresh_typevar(Kind::Type);
    assert_eq!(
        tc.unify_type(
            &UnifyTypeContext {
                expected: Type::Unit,
                actual: Type::Unit,
            },
            v1.clone(),
            Type::mk_arrow(v1.clone(), v2.clone())
        ),
        Err(TypeError::TypeOccurs {
            pos: 0,
            meta: 0,
            ty: Type::mk_arrow(tc.fill_ty_names(v1), tc.fill_ty_names(v2))
        })
    )
}
