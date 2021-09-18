#[cfg(test)]
use std::collections::{HashMap, HashSet};
#[cfg(test)]
use std::rc::Rc;

use crate::typecheck::UnifyKindContextRefs;
#[cfg(test)]
use crate::{
    core::{self, ClassMember, InstanceMember, Placeholder, TypeSig},
    diagnostic::InputLocation,
    evidence::{solver::solve_placeholder, Constraint},
    syntax::{self, Binop, Kind, Spanned, Type},
    typecheck::{BoundVars, TypeError, Typechecker, UnifyKindContext, UnifyTypeContext},
    void::Void,
};

use super::SolveConstraintContext;

#[test]
fn infer_kind_test_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        let expected = Ok((Type::Bool, Kind::Type));
        let actual = tc.infer_kind(&Type::Bool);
        assert_eq!(expected, actual)
    })
}

#[test]
fn infer_kind_test_2() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        let expected = Ok((Type::RowNil, Kind::Row));
        let actual = tc.infer_kind(&Type::RowNil);
        assert_eq!(expected, actual)
    })
}

#[test]
fn infer_kind_test_3() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        let expected = Err(TypeError::KindMismatch {
            location: InputLocation::Interactive {
                label: String::from("(typechecker)"),
            },
            pos: 0,
            context: UnifyKindContext {
                ty: Type::RowNil,
                has_kind: Kind::Type,
                unifying_types: None,
            },
            expected: Kind::Type,
            actual: Kind::Row,
        });
        let actual = tc.infer_kind(&Type::mk_rowcons(Rc::from("x"), Type::RowNil, Type::RowNil));
        assert_eq!(expected, actual)
    })
}

#[test]
fn infer_kind_test_4() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        let expected = Ok(Kind::Type);
        let actual = tc
            .infer_kind(&Type::mk_app(
                Type::Record,
                Type::mk_rowcons(Rc::from("x"), Type::Bool, Type::RowNil),
            ))
            .map(|(_, kind)| tc.zonk_kind(false, &kind));
        assert_eq!(expected, actual)
    })
}

#[test]
fn context_test_1() {
    let mut ctx = BoundVars::new();
    ctx.insert(&vec![
        (Rc::from("a"), Type::Unit::<usize>),
        (Rc::from("b"), Type::Bool),
        (Rc::from("c"), Type::String),
    ]);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![
                (Rc::from("a"), vec![2]),
                (Rc::from("b"), vec![1]),
                (Rc::from("c"), vec![0])
            ]
            .into_iter()
            .collect(),
            info: vec![
                (Rc::from("a"), Type::Unit),
                (Rc::from("b"), Type::Bool),
                (Rc::from("c"), Type::String),
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
        (Rc::from("a"), Type::Unit::<usize>),
        (Rc::from("a"), Type::Bool),
        (Rc::from("c"), Type::String),
    ]);
}

#[test]
fn context_test_3() {
    let mut ctx = BoundVars::new();
    ctx.insert(&[(Rc::from("a"), Type::Unit::<usize>)]);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![(Rc::from("a"), vec![0]),].into_iter().collect(),
            info: vec![(Rc::from("a"), Type::Unit),]
        }
    );
    ctx.insert(&[(Rc::from("b"), Type::Bool)]);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![(Rc::from("a"), vec![1]), (Rc::from("b"), vec![0]),]
                .into_iter()
                .collect(),
            info: vec![(Rc::from("a"), Type::Unit), (Rc::from("b"), Type::Bool),]
        }
    );
    ctx.insert(&[(Rc::from("c"), Type::String)]);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![
                (Rc::from("a"), vec![2]),
                (Rc::from("b"), vec![1]),
                (Rc::from("c"), vec![0])
            ]
            .into_iter()
            .collect(),
            info: vec![
                (Rc::from("a"), Type::Unit),
                (Rc::from("b"), Type::Bool),
                (Rc::from("c"), Type::String),
            ]
        }
    );
}

#[test]
fn context_test_4() {
    let mut ctx = BoundVars::new();
    ctx.insert(&[(Rc::from("a"), Type::Unit::<usize>)]);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![(Rc::from("a"), vec![0]),].into_iter().collect(),
            info: vec![(Rc::from("a"), Type::Unit),]
        }
    );
    ctx.delete(1);
    assert_eq!(ctx, BoundVars::new())
}

#[test]
fn context_test_5() {
    let mut ctx = BoundVars::new();
    ctx.insert(&[(Rc::from("a"), Type::Unit::<usize>)]);
    ctx.insert(&[(Rc::from("b"), Type::Bool)]);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![(Rc::from("a"), vec![1]), (Rc::from("b"), vec![0])]
                .into_iter()
                .collect(),
            info: vec![(Rc::from("a"), Type::Unit), (Rc::from("b"), Type::Bool)]
        }
    );
    ctx.delete(1);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![(Rc::from("a"), vec![0]),].into_iter().collect(),
            info: vec![(Rc::from("a"), Type::Unit),]
        }
    )
}

#[test]
fn context_test_6() {
    let mut ctx = BoundVars::new();
    ctx.insert(&[(Rc::from("a"), Type::Unit::<usize>)]);
    ctx.insert(&[(Rc::from("b"), Type::Bool)]);
    assert_eq!(
        ctx,
        BoundVars {
            indices: vec![(Rc::from("a"), vec![1]), (Rc::from("b"), vec![0])]
                .into_iter()
                .collect(),
            info: vec![(Rc::from("a"), Type::Unit), (Rc::from("b"), Type::Bool)]
        }
    );
    ctx.delete(2);
    assert_eq!(ctx, BoundVars::new())
}

#[test]
fn infer_pattern_test_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        let pat = syntax::Pattern::Name(syntax::Spanned {
            pos: 0,
            item: String::from("x"),
        });
        assert_eq!(
            tc.infer_pattern(&pat),
            (
                core::Pattern::Name,
                syntax::Type::Meta(0),
                vec![(Rc::from("x"), syntax::Type::Meta(0))]
            )
        )
    })
}

#[test]
fn infer_pattern_test_2() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
                        core::Expr::mk_placeholder(2),
                        core::Expr::mk_placeholder(1),
                        core::Expr::mk_placeholder(0)
                    ],
                    rest: false
                },
                syntax::Type::mk_record(
                    vec![
                        (Rc::from("x"), syntax::Type::Meta(0)),
                        (Rc::from("y"), syntax::Type::Meta(1)),
                        (Rc::from("z"), syntax::Type::Meta(2))
                    ],
                    None
                ),
                vec![
                    (Rc::from("x"), syntax::Type::Meta(0)),
                    (Rc::from("y"), syntax::Type::Meta(1)),
                    (Rc::from("z"), syntax::Type::Meta(2)),
                ]
            )
        )
    })
}

#[test]
fn infer_pattern_test_3() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
                        core::Expr::mk_placeholder(2),
                        core::Expr::mk_placeholder(1),
                        core::Expr::mk_placeholder(0)
                    ],
                    rest: true
                },
                syntax::Type::mk_record(
                    vec![
                        (Rc::from("x"), syntax::Type::Meta(0)),
                        (Rc::from("y"), syntax::Type::Meta(1)),
                        (Rc::from("z"), syntax::Type::Meta(2))
                    ],
                    Some(syntax::Type::Meta(3))
                ),
                vec![
                    (Rc::from("x"), syntax::Type::Meta(0)),
                    (Rc::from("y"), syntax::Type::Meta(1)),
                    (Rc::from("z"), syntax::Type::Meta(2)),
                    (
                        Rc::from("w"),
                        syntax::Type::mk_record(Vec::new(), Some(syntax::Type::Meta(3)))
                    ),
                ]
            )
        )
    })
}

#[test]
fn infer_pattern_test_4() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
                core::Pattern::mk_variant(core::Expr::mk_placeholder(0)),
                syntax::Type::mk_variant(
                    vec![(Rc::from("just"), syntax::Type::Meta(0))],
                    Some(syntax::Type::Meta(1))
                ),
                vec![(Rc::from("x"), syntax::Type::Meta(0))]
            )
        )
    })
}

#[test]
fn infer_lam_test_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
        let expected = Ok((
            core::Expr::mk_lam(true, core::Expr::Var(0)),
            syntax::Type::mk_arrow(syntax::Type::Meta(4), syntax::Type::Meta(4)),
        ));
        let actual = tc
            .infer_expr(&term)
            .map(|(expr, ty)| (expr, tc.zonk_type(&ty)));
        assert_eq!(expected, actual)
    })
}

#[test]
fn infer_lam_test_2() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
        let actual = tc
            .infer_expr(&term)
            .map(|(expr, ty)| (expr, tc.zonk_type(&ty)));
        let expected = Ok((
            core::Expr::mk_lam(
                true,
                core::Expr::mk_case(
                    core::Expr::Var(0),
                    vec![core::Branch {
                        pattern: core::Pattern::Record {
                            names: vec![
                                core::Expr::mk_placeholder(1),
                                core::Expr::mk_placeholder(0),
                            ],
                            rest: false,
                        },
                        body: core::Expr::Var(1),
                    }],
                ),
            ),
            syntax::Type::mk_arrow(
                syntax::Type::mk_record(
                    vec![
                        (Rc::from("x"), syntax::Type::Meta(4)),
                        (Rc::from("y"), syntax::Type::Meta(5)),
                    ],
                    None,
                ),
                syntax::Type::Meta(4),
            ),
        ));
        assert_eq!(expected, actual)
    })
}

#[test]
fn infer_lam_test_3() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
        let actual = tc
            .infer_expr(&term)
            .map(|(expr, ty)| (expr, tc.zonk_type(&ty)));
        let expected = Ok((
            core::Expr::mk_lam(
                true,
                core::Expr::mk_case(
                    core::Expr::Var(0),
                    vec![core::Branch {
                        pattern: core::Pattern::Record {
                            names: vec![
                                core::Expr::mk_placeholder(1),
                                core::Expr::mk_placeholder(0),
                            ],
                            rest: false,
                        },
                        body: core::Expr::Var(0),
                    }],
                ),
            ),
            syntax::Type::mk_arrow(
                syntax::Type::mk_record(
                    vec![
                        (Rc::from("x"), syntax::Type::Meta(4)),
                        (Rc::from("y"), syntax::Type::Meta(5)),
                    ],
                    None,
                ),
                syntax::Type::Meta(5),
            ),
        ));
        assert_eq!(expected, actual)
    })
}

#[test]
fn infer_lam_test_4() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
                            names: vec![
                                core::Expr::mk_placeholder(1),
                                core::Expr::mk_placeholder(0),
                            ],
                            rest: true,
                        },
                        body: core::Expr::Var(0),
                    }],
                ),
            ),
            syntax::Type::mk_arrow(
                syntax::Type::mk_record(
                    vec![
                        (Rc::from("x"), syntax::Type::Meta(4)),
                        (Rc::from("y"), syntax::Type::Meta(5)),
                    ],
                    Some(syntax::Type::Meta(6)),
                ),
                syntax::Type::mk_record(vec![], Some(syntax::Type::Meta(6))),
            ),
        ));
        let actual = tc
            .infer_expr(&term)
            .map(|(expr, ty)| (expr, tc.zonk_type(&ty)));
        assert_eq!(expected, actual)
    })
}

#[test]
fn infer_lam_test_5() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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

        let expected = Ok((
            core::Expr::mk_lam(
                true,
                core::Expr::mk_lam(
                    true,
                    core::Expr::mk_app(core::Expr::Var(1), core::Expr::Var(0)),
                ),
            ),
            syntax::Type::mk_arrow(
                syntax::Type::mk_arrow(syntax::Type::Meta(6), syntax::Type::Meta(8)),
                syntax::Type::mk_arrow(syntax::Type::Meta(6), syntax::Type::Meta(8)),
            ),
        ));
        let actual = tc
            .infer_expr(&term)
            .map(|(expr, ty)| (expr, tc.zonk_type(&ty)));
        assert_eq!(expected, actual)
    })
}

#[test]
fn infer_array_test_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
            tc.infer_expr(&term),
            Ok((
                core::Expr::Array(vec![
                    core::Expr::Int(1),
                    core::Expr::Int(2),
                    core::Expr::Int(3)
                ]),
                syntax::Type::mk_app(syntax::Type::Array, syntax::Type::Int)
            ))
        )
    })
}

#[test]
fn infer_array_test_2() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
            tc.infer_expr(&term),
            Err(TypeError::TypeMismatch {
                location: InputLocation::Interactive {
                    label: String::from("(typechecker)"),
                },
                pos: 4,
                context: UnifyTypeContext {
                    expected: syntax::Type::Int,
                    actual: syntax::Type::Bool,
                },
                expected: syntax::Type::Int,
                actual: syntax::Type::Bool
            })
        )
    })
}

#[test]
fn unify_rows_test_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        assert_eq!(
            tc.unify_type(
                &UnifyTypeContext {
                    expected: syntax::Type::Unit,
                    actual: syntax::Type::Unit
                },
                &Type::mk_record(
                    vec![(Rc::from("x"), Type::Int), (Rc::from("y"), Type::Bool)],
                    None
                ),
                &Type::mk_record(
                    vec![(Rc::from("y"), Type::Bool), (Rc::from("x"), Type::Int)],
                    None
                )
            ),
            Ok(())
        )
    })
}

#[test]
fn unify_rows_test_2() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        assert_eq!(
            tc.unify_type(
                &UnifyTypeContext {
                    expected: syntax::Type::Unit,
                    actual: syntax::Type::Unit,
                },
                &Type::mk_record(
                    vec![
                        (Rc::from("x"), Type::Int),
                        (Rc::from("x"), Type::Bool),
                        (Rc::from("y"), Type::Bool)
                    ],
                    None
                ),
                &Type::mk_record(
                    vec![
                        (Rc::from("y"), Type::Bool),
                        (Rc::from("x"), Type::Int),
                        (Rc::from("x"), Type::Bool)
                    ],
                    None
                )
            ),
            Ok(())
        )
    })
}

#[test]
fn unify_rows_test_3() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        assert_eq!(
            tc.unify_type(
                &UnifyTypeContext {
                    expected: syntax::Type::Unit,
                    actual: syntax::Type::Unit,
                },
                &Type::mk_record(
                    vec![
                        (Rc::from("x"), Type::Int),
                        (Rc::from("x"), Type::Bool),
                        (Rc::from("y"), Type::Bool)
                    ],
                    None
                ),
                &Type::mk_record(
                    vec![
                        (Rc::from("x"), Type::Int),
                        (Rc::from("y"), Type::Bool),
                        (Rc::from("x"), Type::Bool)
                    ],
                    None
                )
            ),
            Ok(())
        )
    })
}

#[test]
fn unify_rows_test_4() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        assert_eq!(
            tc.unify_type(
                &UnifyTypeContext {
                    expected: syntax::Type::Unit,
                    actual: syntax::Type::Unit
                },
                &Type::mk_record(
                    vec![
                        (Rc::from("x"), Type::Int),
                        (Rc::from("x"), Type::Bool),
                        (Rc::from("y"), Type::Bool)
                    ],
                    None
                ),
                &Type::mk_record(
                    vec![
                        (Rc::from("x"), Type::Int),
                        (Rc::from("y"), Type::Bool),
                        (Rc::from("x"), Type::Int)
                    ],
                    None
                )
            ),
            Err(TypeError::TypeMismatch {
                location: InputLocation::Interactive {
                    label: String::from("(typechecker)"),
                },
                pos: 0,
                context: UnifyTypeContext {
                    expected: syntax::Type::Unit,
                    actual: syntax::Type::Unit
                },
                expected: Type::Bool,
                actual: Type::Int
            })
        )
    })
}

#[test]
fn infer_record_test_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        // {}
        let term = syntax::Expr::mk_record(Vec::new(), None);
        assert_eq!(
            tc.infer_expr(&syntax::Spanned { pos: 0, item: term })
                .map(|(expr, ty)| (expr, tc.zonk_type(&ty))),
            Ok((
                core::Expr::mk_record(Vec::new(), None),
                syntax::Type::mk_record(Vec::new(), None)
            ))
        )
    })
}

#[test]
fn infer_record_test_2() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
            tc.infer_expr(&syntax::Spanned { pos: 0, item: term })
                .map(|(expr, ty)| (expr, tc.zonk_type(&ty))),
            Ok((
                core::Expr::mk_record(
                    vec![
                        (core::Expr::mk_placeholder(1), core::Expr::Int(1)),
                        (core::Expr::mk_placeholder(0), core::Expr::True)
                    ],
                    None
                ),
                syntax::Type::mk_record(
                    vec![
                        (Rc::from("x"), syntax::Type::Int),
                        (Rc::from("y"), syntax::Type::Bool)
                    ],
                    None
                )
            ))
        )
    })
}

#[test]
fn infer_record_test_3() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
            tc.infer_expr(&syntax::Spanned { pos: 0, item: term })
                .map(|(expr, ty)| (expr, tc.zonk_type(&ty))),
            Ok((
                core::Expr::mk_record(
                    vec![
                        (core::Expr::mk_placeholder(1), core::Expr::Int(1)),
                        (core::Expr::mk_placeholder(0), core::Expr::True)
                    ],
                    Some(core::Expr::mk_record(
                        vec![(core::Expr::mk_placeholder(2), core::Expr::Char('c'))],
                        None
                    ))
                ),
                syntax::Type::mk_record(
                    vec![
                        (Rc::from("x"), syntax::Type::Int),
                        (Rc::from("y"), syntax::Type::Bool),
                        (Rc::from("z"), syntax::Type::Char)
                    ],
                    None
                )
            ))
        )
    })
}

#[test]
fn infer_record_test_4() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
            tc.infer_expr(&syntax::Spanned { pos: 0, item: term })
                .map(|(expr, ty)| (expr, tc.zonk_type(&ty))),
            Err(TypeError::TypeMismatch {
                location: InputLocation::Interactive {
                    label: String::from("(typechecker)"),
                },
                pos: 22,
                context: UnifyTypeContext {
                    expected: syntax::Type::mk_record(Vec::new(), Some(syntax::Type::Meta(0))),
                    actual: syntax::Type::Int
                },
                expected: syntax::Type::mk_record(Vec::new(), Some(syntax::Type::Meta(0))),
                actual: syntax::Type::Int
            })
        )
    })
}

#[test]
fn infer_case_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
        let expected = Ok((
            core::Expr::mk_lam(
                true,
                core::Expr::mk_case(
                    core::Expr::Var(0),
                    vec![core::Branch {
                        pattern: core::Pattern::mk_variant(core::Expr::mk_placeholder(0)),
                        body: core::Expr::Var(0),
                    }],
                ),
            ),
            syntax::Type::mk_arrow(
                syntax::Type::mk_variant(vec![(Rc::from("X"), syntax::Type::Meta(6))], None),
                syntax::Type::Meta(6),
            ),
        ));
        let actual = tc
            .infer_expr(&term)
            .map(|(expr, ty)| (expr, tc.zonk_type(&ty)));
        assert_eq!(expected, actual)
    })
}

#[test]
fn infer_case_2() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
        let expected = Ok((
            core::Expr::mk_lam(
                true,
                core::Expr::mk_case(
                    core::Expr::Var(0),
                    vec![
                        core::Branch {
                            pattern: core::Pattern::mk_variant(core::Expr::mk_placeholder(0)),
                            body: core::Expr::Var(0),
                        },
                        core::Branch {
                            pattern: core::Pattern::mk_variant(core::Expr::mk_placeholder(1)),
                            body: core::Expr::Var(0),
                        },
                    ],
                ),
            ),
            syntax::Type::mk_arrow(
                syntax::Type::mk_variant(
                    vec![
                        (Rc::from("Left"), syntax::Type::Meta(8)),
                        (Rc::from("Right"), syntax::Type::Meta(8)),
                    ],
                    None,
                ),
                syntax::Type::Meta(8),
            ),
        ));
        let actual = tc
            .infer_expr(&term)
            .map(|(expr, ty)| (expr, tc.zonk_type(&ty)));
        assert_eq!(expected, actual)
    })
}

#[test]
fn infer_case_3() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
        let expected = Ok((
            core::Expr::mk_lam(
                true,
                core::Expr::mk_case(
                    core::Expr::Var(0),
                    vec![
                        core::Branch {
                            pattern: core::Pattern::mk_variant(core::Expr::mk_placeholder(0)),
                            body: core::Expr::Var(0),
                        },
                        core::Branch {
                            pattern: core::Pattern::mk_variant(core::Expr::mk_placeholder(1)),
                            body: core::Expr::Var(0),
                        },
                        core::Branch {
                            pattern: core::Pattern::Wildcard,
                            body: core::Expr::Int(1),
                        },
                    ],
                ),
            ),
            syntax::Type::mk_arrow(
                syntax::Type::mk_variant(
                    vec![
                        (Rc::from("Left"), syntax::Type::Int),
                        (Rc::from("Right"), syntax::Type::Int),
                    ],
                    Some(syntax::Type::Meta(9)),
                ),
                syntax::Type::Int,
            ),
        ));
        let actual = tc
            .infer_expr(&term)
            .map(|(expr, ty)| (expr, tc.zonk_type(&ty)));
        assert_eq!(expected, actual)
    })
}

#[test]
fn infer_case_4() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
            tc.infer_expr(&term)
                .map(|(expr, ty)| (expr, tc.zonk_type(&ty))),
            Err(TypeError::RedundantPattern {
                location: InputLocation::Interactive {
                    label: String::from("(typechecker)"),
                },
                pos: 32
            })
        )
    })
}

#[test]
fn infer_record_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        let expected_expr = core::Expr::mk_record(
            vec![
                (core::Expr::Placeholder(Placeholder(2)), core::Expr::False),
                (
                    core::Expr::Placeholder(Placeholder(1)),
                    core::Expr::String(Vec::new()),
                ),
                (core::Expr::Placeholder(Placeholder(0)), core::Expr::Int(0)),
            ],
            None,
        );
        let expected_ty = Type::mk_record(
            vec![
                (Rc::from("z"), Type::Bool),
                (Rc::from("y"), Type::String),
                (Rc::from("x"), Type::Int),
            ],
            None,
        );
        let expected_result = Ok((expected_expr, expected_ty));
        // { z = False, y = "", x = 0 }
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
            .infer_expr(&expr)
            .map(|(expr, ty)| (expr, tc.zonk_type(&ty)));
        assert_eq!(expected_result, actual_result, "checking results");

        let (actual_expr, _actual_ty) = actual_result.unwrap();

        let mut placeholders: HashSet<Placeholder> = HashSet::new();
        let _: Result<core::Expr, Void> = actual_expr.subst_placeholder(&mut |p| {
            placeholders.insert(*p);
            Ok(core::Expr::Placeholder(*p))
        });

        assert_eq!(3, placeholders.len(), "checking number of Placeholders");

        let p0 = placeholders.get(&Placeholder(0)).unwrap();
        let p1 = placeholders.get(&Placeholder(1)).unwrap();
        let p2 = placeholders.get(&Placeholder(2)).unwrap();

        assert_eq!(
            Ok((
                core::Expr::Int(0),
                Constraint::HasField {
                    field: Rc::from("x"),
                    rest: Type::RowNil
                }
            )),
            solve_placeholder(&mut tc, *p0)
                .map(|(expr, constraint)| (expr, tc.zonk_constraint(&constraint)))
        );

        assert_eq!(
            Ok((
                core::Expr::mk_binop(Binop::Add, core::Expr::Int(1), core::Expr::Int(0)),
                Constraint::HasField {
                    field: Rc::from("y"),
                    rest: Type::mk_rows(vec![(Rc::from("x"), Type::Int)], None)
                }
            )),
            solve_placeholder(&mut tc, *p1)
                .map(|(expr, constraint)| (expr, tc.zonk_constraint(&constraint)))
        );

        assert_eq!(
            Ok((
                core::Expr::mk_binop(
                    Binop::Add,
                    core::Expr::Int(1),
                    core::Expr::mk_binop(Binop::Add, core::Expr::Int(1), core::Expr::Int(0))
                ),
                Constraint::HasField {
                    field: Rc::from("z"),
                    rest: Type::mk_rows(
                        vec![(Rc::from("y"), Type::String), (Rc::from("x"), Type::Int)],
                        None
                    )
                }
            )),
            solve_placeholder(&mut tc, *p2)
                .map(|(expr, constraint)| (expr, tc.zonk_constraint(&constraint)))
        );
    })
}

#[test]
fn check_definition_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        /*
        id : a -> a
        id x = x
        */
        let decl = syntax::Spanned {
            pos: 0,
            item: syntax::Declaration::Definition {
                name: String::from("id"),
                ty: syntax::Type::mk_arrow(
                    syntax::Type::Var(Rc::from("a")),
                    syntax::Type::Var(Rc::from("a")),
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
            tc.check_declaration(&mut HashMap::new(), &decl),
            Ok(Some(core::Declaration::Definition {
                name: String::from("id"),
                sig: core::TypeSig {
                    ty_vars: vec![(Rc::from("a"), syntax::Kind::Type)],
                    body: syntax::Type::mk_arrow(syntax::Type::Var(0), syntax::Type::Var(0))
                },
                body: core::Expr::mk_lam(true, core::Expr::Var(0))
            }))
        )
    })
}

#[test]
fn check_definition_2() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        /*
        thing : { r } -> { x : Int, r }
        thing r = { x = 0, ..r }
        */
        let decl = syntax::Spanned {
            pos: 0,
            item: syntax::Declaration::Definition {
                name: String::from("thing"),
                ty: syntax::Type::mk_arrow(
                    syntax::Type::mk_record(Vec::new(), Some(Type::Var(Rc::from("r")))),
                    syntax::Type::mk_record(
                        vec![(Rc::from("x"), Type::Int)],
                        Some(syntax::Type::Var(Rc::from("r"))),
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
        let expected = Ok(Some(core::Declaration::Definition {
            name: String::from("thing"),
            sig: core::TypeSig {
                ty_vars: vec![(Rc::from("r"), syntax::Kind::Row)],
                body: syntax::Type::mk_fatarrow(
                    syntax::Type::mk_hasfield(Rc::from("x"), Type::Var(0)),
                    syntax::Type::mk_arrow(
                        syntax::Type::mk_record(Vec::new(), Some(Type::Var(0))),
                        syntax::Type::mk_record(
                            vec![(Rc::from("x"), Type::Int)],
                            Some(syntax::Type::Var(0)),
                        ),
                    ),
                ),
            },
            body: core::Expr::mk_lam(
                true,
                core::Expr::mk_lam(
                    true,
                    core::Expr::mk_extend(
                        core::Expr::Var(1),
                        core::Expr::Int(0),
                        core::Expr::Var(0),
                    ),
                ),
            ),
        }));
        let actual = tc.check_declaration(&mut HashMap::new(), &decl);
        assert_eq!(expected, actual)
    })
}

#[test]
fn check_definition_3() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
                        (Rc::from("z"), Type::Bool),
                        (Rc::from("y"), Type::String),
                        (Rc::from("x"), Type::Int),
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
        let expected = Ok(Some(core::Declaration::Definition {
            name: String::from("thing"),
            sig: core::TypeSig {
                ty_vars: Vec::new(),
                body: syntax::Type::mk_record(
                    vec![
                        (Rc::from("z"), Type::Bool),
                        (Rc::from("y"), Type::String),
                        (Rc::from("x"), Type::Int),
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
        }));
        let actual = tc.check_declaration(&mut HashMap::new(), &decl);
        assert_eq!(expected, actual)
    })
}

#[test]
fn check_definition_4() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
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
                        vec![(Rc::from("x"), Type::Int)],
                        Some(syntax::Type::Var(Rc::from("r"))),
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
        let expected = Ok(Some(core::Declaration::Definition {
            name: String::from("getx"),
            sig: core::TypeSig {
                ty_vars: vec![(Rc::from("r"), syntax::Kind::Row)],
                body: syntax::Type::mk_fatarrow(
                    syntax::Type::mk_hasfield(Rc::from("x"), syntax::Type::Var(0)),
                    syntax::Type::mk_arrow(
                        syntax::Type::mk_record(
                            vec![(Rc::from("x"), syntax::Type::Int)],
                            Some(syntax::Type::Var(0)),
                        ),
                        syntax::Type::Int,
                    ),
                ),
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
                                rest: true,
                            },
                            body: core::Expr::Var(1),
                        }],
                    ),
                ),
            ),
        }));
        let actual = tc.check_declaration(&mut HashMap::new(), &decl);
        assert_eq!(expected, actual)
    })
}

#[test]
fn kind_occurs_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        let v1 = tc.fresh_kindvar();
        let v2 = tc.fresh_kindvar();
        assert_eq!(
            tc.unify_kind(
                &UnifyKindContextRefs {
                    ty: &Type::Unit,
                    has_kind: &Kind::Type,
                    unifying_types: None
                },
                &v1,
                &Kind::mk_arrow(v1.clone(), v2.clone())
            ),
            Err(TypeError::KindOccurs {
                location: InputLocation::Interactive {
                    label: String::from("(typechecker)"),
                },
                pos: 0,
                meta: 0,
                kind: Kind::mk_arrow(v1, v2)
            })
        )
    })
}

#[test]
fn type_occurs_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        let v1 = tc.fresh_typevar(Kind::Type);
        let v2 = tc.fresh_typevar(Kind::Type);
        assert_eq!(
            tc.unify_type(
                &UnifyTypeContext {
                    expected: Type::Unit,
                    actual: Type::Unit,
                },
                &v1,
                &Type::mk_arrow(v1.clone(), v2.clone())
            ),
            Err(TypeError::TypeOccurs {
                location: InputLocation::Interactive {
                    label: String::from("(typechecker)"),
                },
                pos: 0,
                meta: 0,
                ty: Type::mk_arrow(tc.fill_ty_names(v1), tc.fill_ty_names(v2))
            })
        )
    })
}

#[test]
fn check_class_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        let expected = Ok(Some(core::Declaration::Class(core::ClassDeclaration {
            supers: Vec::new(),
            name: Rc::from("Eq"),
            args: vec![(Rc::from("a"), Kind::Type)],
            members: vec![ClassMember {
                name: String::from("eq"),
                sig: TypeSig {
                    ty_vars: vec![(Rc::from("a"), Kind::Type)],
                    body: Type::mk_arrow(Type::Var(0), Type::mk_arrow(Type::Var(0), Type::Bool)),
                },
            }],
        })));
        /*
        class Eq a where
          eq : a -> a -> Bool
        */
        let actual = tc.check_declaration(
            &mut HashMap::new(),
            &Spanned {
                pos: 0,
                item: syntax::Declaration::Class {
                    supers: Vec::new(),
                    name: Rc::from("Eq"),
                    args: vec![Spanned {
                        pos: 9,
                        item: Rc::from("a"),
                    }],
                    members: vec![(
                        String::from("eq"),
                        Type::mk_arrow(
                            Type::Var(Rc::from("a")),
                            Type::mk_arrow(Type::Var(Rc::from("a")), Type::Bool),
                        ),
                    )],
                },
            },
        );
        assert_eq!(expected, actual);

        let decl = actual.unwrap().unwrap();
        tc.register_declaration(&decl);

        let expected_context: HashMap<Rc<str>, core::ClassDeclaration> = vec![(
            Rc::from("Eq"),
            core::ClassDeclaration {
                supers: Vec::new(),
                args: vec![(Rc::from("a"), Kind::Type)],
                name: Rc::from("Eq"),
                members: vec![core::ClassMember {
                    name: String::from("eq"),
                    sig: core::TypeSig {
                        ty_vars: vec![(Rc::from("a"), Kind::Type)],
                        body: Type::mk_arrow(
                            Type::Var(0),
                            Type::mk_arrow(Type::Var(0), Type::Bool),
                        ),
                    },
                }],
            },
        )]
        .into_iter()
        .collect();

        assert_eq!(expected_context, tc.class_context);

        let expected_member = (
            core::TypeSig {
                ty_vars: vec![(Rc::from("a"), Kind::Type)],
                body: Type::mk_fatarrow(
                    Type::mk_app(Type::Name(Rc::from("Eq")), Type::Var(0)),
                    Type::mk_arrow(Type::Var(0), Type::mk_arrow(Type::Var(0), Type::Bool)),
                ),
            },
            core::Expr::mk_lam(
                true,
                core::Expr::mk_project(core::Expr::Var(0), core::Expr::Int(0)),
            ),
        );
        assert_eq!(
            Some(&expected_member),
            tc.registered_bindings.get(&String::from("eq"))
        );
    })
}

#[test]
fn check_class_2() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        let expected = Ok(Some(core::Declaration::Class(core::ClassDeclaration {
            supers: Vec::new(),
            name: Rc::from("Wut"),
            args: vec![(Rc::from("a"), Kind::Type)],
            members: vec![ClassMember {
                name: String::from("wut"),
                sig: TypeSig {
                    ty_vars: vec![(Rc::from("a"), Kind::Type), (Rc::from("b"), Kind::Type)],
                    body: Type::mk_arrow(Type::Var(1), Type::mk_arrow(Type::Var(0), Type::Bool)),
                },
            }],
        })));
        /*
        class Wut a where
          wut : a -> b -> Bool
        */
        let actual = tc.check_declaration(
            &mut HashMap::new(),
            &Spanned {
                pos: 0,
                item: syntax::Declaration::Class {
                    supers: Vec::new(),
                    name: Rc::from("Wut"),
                    args: vec![Spanned {
                        pos: 9,
                        item: Rc::from("a"),
                    }],
                    members: vec![(
                        String::from("wut"),
                        Type::mk_arrow(
                            Type::Var(Rc::from("a")),
                            Type::mk_arrow(Type::Var(Rc::from("b")), Type::Bool),
                        ),
                    )],
                },
            },
        );
        assert_eq!(expected, actual);

        let decl = actual.unwrap().unwrap();
        tc.register_declaration(&decl);

        let expected_context: HashMap<Rc<str>, core::ClassDeclaration> = vec![(
            Rc::from("Wut"),
            core::ClassDeclaration {
                supers: Vec::new(),
                args: vec![(Rc::from("a"), Kind::Type)],
                name: Rc::from("Wut"),
                members: vec![core::ClassMember {
                    name: String::from("wut"),
                    sig: core::TypeSig {
                        ty_vars: vec![(Rc::from("a"), Kind::Type), (Rc::from("b"), Kind::Type)],
                        body: Type::mk_arrow(
                            Type::Var(1),
                            Type::mk_arrow(Type::Var(0), Type::Bool),
                        ),
                    },
                }],
            },
        )]
        .into_iter()
        .collect();

        assert_eq!(expected_context, tc.class_context);

        let expected_member = (
            core::TypeSig {
                ty_vars: vec![(Rc::from("a"), Kind::Type), (Rc::from("b"), Kind::Type)],
                body: Type::mk_fatarrow(
                    Type::mk_app(Type::Name(Rc::from("Wut")), Type::Var(1)),
                    Type::mk_arrow(Type::Var(1), Type::mk_arrow(Type::Var(0), Type::Bool)),
                ),
            },
            core::Expr::mk_lam(
                true,
                core::Expr::mk_project(core::Expr::Var(0), core::Expr::Int(0)),
            ),
        );
        assert_eq!(
            Some(&expected_member),
            tc.registered_bindings.get(&String::from("wut")),
            "expected member"
        );
    })
}

#[test]
fn check_instance_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        let expected = Ok(Some(core::Declaration::Instance {
            ty_vars: Vec::new(),
            superclass_constructors: Vec::new(),
            assumes: Vec::new(),
            head: Type::mk_app(Type::Name(Rc::from("Eq")), Type::Unit),
            members: vec![InstanceMember {
                name: String::from("eq"),
                body: core::Expr::mk_lam(true, core::Expr::mk_lam(true, core::Expr::True)),
            }],
        }));
        tc.register_declaration(&core::Declaration::Class(core::ClassDeclaration {
            supers: Vec::new(),
            name: Rc::from("Eq"),
            args: vec![(Rc::from("a"), Kind::Type)],
            members: vec![ClassMember {
                name: String::from("eq"),
                sig: TypeSig {
                    ty_vars: vec![(Rc::from("a"), Kind::Type)],
                    body: Type::mk_arrow(Type::Var(0), Type::mk_arrow(Type::Var(0), Type::Bool)),
                },
            }],
        }));
        /*
        instance Eq () where
          eq x y = True
        */
        let actual = tc.check_declaration(
            &mut HashMap::new(),
            &Spanned {
                pos: 0,
                item: syntax::Declaration::Instance {
                    assumes: Vec::new(),
                    name: Spanned {
                        pos: 9,
                        item: Rc::from("Eq"),
                    },
                    args: vec![Type::Unit],
                    members: vec![(
                        Spanned {
                            pos: 22,
                            item: String::from("eq"),
                        },
                        vec![
                            syntax::Pattern::Name(Spanned {
                                pos: 25,
                                item: String::from("x"),
                            }),
                            syntax::Pattern::Name(Spanned {
                                pos: 27,
                                item: String::from("y"),
                            }),
                        ],
                        Spanned {
                            pos: 31,
                            item: syntax::Expr::True,
                        },
                    )],
                },
            },
        );
        assert_eq!(expected, actual)
    })
}

#[test]
fn class_and_instance_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        /*
        class Eq a where
          eq : a -> a -> Bool

        class Eq a => Ord a where
          lt : a -> a -> Bool

        instance Eq Int where
          eq = eqInt

        instance Ord Int where
          lt = ltInt

        eqDictInt = {
            eq = eqInt
        }

        ordDictInt = {
            eqDict = eqDictInt
            lt = ltInt
        }
         */

        tc.register_class(&core::ClassDeclaration {
            supers: Vec::new(),
            name: Rc::from("Eq"),
            args: vec![(Rc::from("a"), Kind::Type)],
            members: vec![core::ClassMember {
                name: String::from("eq"),
                sig: core::TypeSig {
                    ty_vars: vec![(Rc::from("a"), Kind::Type)],
                    body: Type::mk_arrow(Type::Var(0), Type::mk_arrow(Type::Var(0), Type::Bool)),
                },
            }],
        });

        tc.register_class(&core::ClassDeclaration {
            supers: vec![Type::mk_app(Type::Name(Rc::from("Eq")), Type::Var(0))],
            name: Rc::from("Ord"),
            args: vec![(Rc::from("a"), Kind::Type)],
            members: vec![core::ClassMember {
                name: String::from("lt"),
                sig: core::TypeSig {
                    ty_vars: vec![(Rc::from("a"), Kind::Type)],
                    body: Type::mk_arrow(Type::Var(0), Type::mk_arrow(Type::Var(0), Type::Bool)),
                },
            }],
        });

        let instance_eq_int_decl = Spanned {
            pos: 0,
            item: syntax::Declaration::Instance {
                assumes: Vec::new(),
                name: Spanned {
                    pos: 0,
                    item: Rc::from("Eq"),
                },
                args: vec![Type::Int],
                members: vec![(
                    Spanned {
                        pos: 0,
                        item: String::from("eq"),
                    },
                    Vec::new(),
                    Spanned {
                        pos: 0,
                        item: syntax::Expr::Var(String::from("eqInt")),
                    },
                )],
            },
        };

        let instance_ord_int_decl = Spanned {
            pos: 0,
            item: syntax::Declaration::Instance {
                assumes: Vec::new(),
                name: Spanned {
                    pos: 0,
                    item: Rc::from("Ord"),
                },
                args: vec![Type::Int],
                members: vec![(
                    Spanned {
                        pos: 0,
                        item: String::from("lt"),
                    },
                    Vec::new(),
                    Spanned {
                        pos: 0,
                        item: syntax::Expr::Var(String::from("ltInt")),
                    },
                )],
            },
        };

        let expected_instance_ord_int_result = Err(TypeError::CannotDeduce {
            location: InputLocation::Interactive {
                label: String::from("(typechecker)"),
            },
            context: Some(SolveConstraintContext {
                pos: 0,
                constraint: Type::mk_app(Type::Name(Rc::from("Eq")), Type::Int),
            }),
        });
        let actual_instance_ord_int_result =
            tc.check_declaration(&mut HashMap::new(), &instance_ord_int_decl.clone());

        assert_eq!(
        expected_instance_ord_int_result,
        actual_instance_ord_int_result,
        "When `instance Eq Int` is not in scope, `instance Ord Int` fails to type check because `Eq` is a superclass of `Ord`"
    );

        let expected_instance_eq_int_result = Ok(Some(core::Declaration::Instance {
            ty_vars: Vec::new(),
            superclass_constructors: Vec::new(),
            assumes: Vec::new(),
            head: Type::mk_app(Type::Name(Rc::from("Eq")), Type::Int),
            members: vec![core::InstanceMember {
                name: String::from("eq"),
                body: core::Expr::Name(String::from("eqInt")),
            }],
        }));
        let actual_instance_eq_int_result =
            tc.check_declaration(&mut HashMap::new(), &instance_eq_int_decl);

        assert_eq!(
            expected_instance_eq_int_result, actual_instance_eq_int_result,
            "`instance Eq Int` is valid"
        );

        tc.register_declaration(&actual_instance_eq_int_result.unwrap().unwrap());

        let expected_instance_ord_int_result = Ok(Some(core::Declaration::Instance {
            ty_vars: Vec::new(),
            superclass_constructors: vec![core::Expr::mk_record(
                vec![(core::Expr::Int(0), core::Expr::Name(String::from("eqInt")))],
                None,
            )],
            assumes: Vec::new(),
            head: Type::mk_app(Type::Name(Rc::from("Ord")), Type::Int),
            members: vec![core::InstanceMember {
                name: String::from("lt"),
                body: core::Expr::Name(String::from("ltInt")),
            }],
        }));
        let actual_instance_ord_int_result =
            tc.check_declaration(&mut HashMap::new(), &instance_ord_int_decl);

        assert_eq!(
            expected_instance_ord_int_result, actual_instance_ord_int_result,
            "After `instance Eq Int` is brought into scope, `instance Ord Int` is valid"
        );
    })
}

#[test]
fn class_and_instance_2() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        /*
        class Eq a where
          eq : a -> a -> Bool

        instance Eq Int where
          eq = eqInt

        instance Eq a => Eq (Array a) where
          eq = eqArray eq

        instance Ord a => Ord (Array a) where
          lt = ltArray lt

        instance Ord Int where
          lt = ltInt

        comparison : Bool
        comparison = lt [0, 1, 2] [3, 4]
         */

        tc.register_class(&core::ClassDeclaration {
            supers: Vec::new(),
            name: Rc::from("Eq"),
            args: vec![(Rc::from("a"), Kind::Type)],
            members: vec![core::ClassMember {
                name: String::from("eq"),
                sig: core::TypeSig {
                    ty_vars: vec![(Rc::from("a"), Kind::Type)],
                    body: Type::mk_arrow(Type::Var(0), Type::mk_arrow(Type::Var(0), Type::Bool)),
                },
            }],
        });

        tc.register_class(&core::ClassDeclaration {
            supers: vec![Type::mk_app(Type::Name(Rc::from("Eq")), Type::Var(0))],
            name: Rc::from("Ord"),
            args: vec![(Rc::from("a"), Kind::Type)],
            members: vec![core::ClassMember {
                name: String::from("lt"),
                sig: core::TypeSig {
                    ty_vars: vec![(Rc::from("a"), Kind::Type)],
                    body: Type::mk_arrow(Type::Var(0), Type::mk_arrow(Type::Var(0), Type::Bool)),
                },
            }],
        });

        let instance_eq_int_decl = Spanned {
            pos: 0,
            item: syntax::Declaration::Instance {
                assumes: Vec::new(),
                name: Spanned {
                    pos: 0,
                    item: Rc::from("Eq"),
                },
                args: vec![Type::Int],
                members: vec![(
                    Spanned {
                        pos: 0,
                        item: String::from("eq"),
                    },
                    Vec::new(),
                    Spanned {
                        pos: 0,
                        item: syntax::Expr::Var(String::from("eqInt")),
                    },
                )],
            },
        };

        let instance_eq_array_decl = Spanned {
            pos: 0,
            item: syntax::Declaration::Instance {
                assumes: vec![Spanned {
                    pos: 0,
                    item: Type::mk_app(Type::Name(Rc::from("Eq")), Type::Var(Rc::from("a"))),
                }],
                name: Spanned {
                    pos: 0,
                    item: Rc::from("Eq"),
                },
                args: vec![Type::mk_app(Type::Array, Type::Var(Rc::from("a")))],
                members: vec![(
                    Spanned {
                        pos: 0,
                        item: String::from("eq"),
                    },
                    Vec::new(),
                    syntax::Expr::mk_app(
                        Spanned {
                            pos: 0,
                            item: syntax::Expr::Var(String::from("eqArray")),
                        },
                        Spanned {
                            pos: 0,
                            item: syntax::Expr::Var(String::from("eq")),
                        },
                    ),
                )],
            },
        };

        let expected_instance_eq_int_result = Ok(Some(core::Declaration::Instance {
            ty_vars: Vec::new(),
            superclass_constructors: Vec::new(),
            assumes: Vec::new(),
            head: Type::mk_app(Type::Name(Rc::from("Eq")), Type::Int),
            members: vec![core::InstanceMember {
                name: String::from("eq"),
                body: core::Expr::Name(String::from("eqInt")),
            }],
        }));
        let actual_instance_eq_int_result =
            tc.check_declaration(&mut HashMap::new(), &instance_eq_int_decl);

        assert_eq!(
            expected_instance_eq_int_result, actual_instance_eq_int_result,
            "`instance Eq Int` is valid"
        );

        tc.register_declaration(&actual_instance_eq_int_result.unwrap().unwrap());

        let expected_instance_eq_array_result = Ok(Some(core::Declaration::Instance {
            ty_vars: vec![(Rc::from("a"), Kind::Type)],
            superclass_constructors: Vec::new(),
            assumes: vec![Type::mk_app(Type::Name(Rc::from("Eq")), Type::Var(0))],
            head: Type::mk_app(
                Type::Name(Rc::from("Eq")),
                Type::mk_app(Type::Array, Type::Var(0)),
            ),
            members: vec![core::InstanceMember {
                name: String::from("eq"),
                body: core::Expr::mk_lam(
                    true,
                    core::Expr::mk_app(
                        core::Expr::Name(String::from("eqArray")),
                        core::Expr::mk_app(
                            core::Expr::Name(String::from("eq")),
                            core::Expr::Var(0),
                        ),
                    ),
                ),
            }],
        }));
        let actual_instance_eq_array_result =
            tc.check_declaration(&mut HashMap::new(), &instance_eq_array_decl);

        assert_eq!(
            expected_instance_eq_array_result, actual_instance_eq_array_result,
            "`instance Eq a => Eq (Array a)` is valid"
        );

        tc.register_declaration(&actual_instance_eq_array_result.unwrap().unwrap());

        let instance_ord_array_decl = Spanned {
            pos: 0,
            item: syntax::Declaration::Instance {
                assumes: vec![Spanned {
                    pos: 0,
                    item: Type::mk_app(Type::Name(Rc::from("Ord")), Type::Var(Rc::from("a"))),
                }],
                name: Spanned {
                    pos: 0,
                    item: Rc::from("Ord"),
                },
                args: vec![Type::mk_app(Type::Array, Type::Var(Rc::from("a")))],
                members: vec![(
                    Spanned {
                        pos: 0,
                        item: String::from("lt"),
                    },
                    Vec::new(),
                    syntax::Expr::mk_app(
                        Spanned {
                            pos: 0,
                            item: syntax::Expr::Var(String::from("ltArray")),
                        },
                        Spanned {
                            pos: 0,
                            item: syntax::Expr::Var(String::from("lt")),
                        },
                    ),
                )],
            },
        };

        let expected_instance_ord_array_result = Ok(Some(core::Declaration::Instance {
            ty_vars: vec![(Rc::from("a"), Kind::Type)],
            superclass_constructors: vec![core::Expr::mk_lam(
                true, // dict : Ord a
                core::Expr::mk_record(
                    vec![(
                        core::Expr::Int(0),
                        core::Expr::mk_app(
                            core::Expr::Name(String::from("eqArray")),
                            core::Expr::mk_app(
                                core::Expr::Name(String::from("eq")),
                                // dict.0 : Eq a
                                core::Expr::mk_project(core::Expr::Var(0), core::Expr::Int(0)),
                            ),
                        ),
                    )],
                    None,
                ),
            )],
            assumes: vec![Type::mk_app(Type::Name(Rc::from("Ord")), Type::Var(0))],
            head: Type::mk_app(
                Type::Name(Rc::from("Ord")),
                Type::mk_app(Type::Array, Type::Var(0)),
            ),
            members: vec![core::InstanceMember {
                name: String::from("lt"),
                body: core::Expr::mk_lam(
                    true,
                    core::Expr::mk_app(
                        core::Expr::Name(String::from("ltArray")),
                        core::Expr::mk_app(
                            core::Expr::Name(String::from("lt")),
                            core::Expr::Var(0),
                        ),
                    ),
                ),
            }],
        }));
        let actual_instance_ord_array_result =
            tc.check_declaration(&mut HashMap::new(), &instance_ord_array_decl);

        assert_eq!(
            expected_instance_ord_array_result, actual_instance_ord_array_result,
            "`instance Ord a => Ord (Array a)` is valid"
        );

        tc.register_declaration(&actual_instance_ord_array_result.unwrap().unwrap());

        let instance_ord_int_decl = Spanned {
            pos: 0,
            item: syntax::Declaration::Instance {
                assumes: Vec::new(),
                name: Spanned {
                    pos: 0,
                    item: Rc::from("Ord"),
                },
                args: vec![Type::Int],
                members: vec![(
                    Spanned {
                        pos: 0,
                        item: String::from("lt"),
                    },
                    Vec::new(),
                    Spanned {
                        pos: 0,
                        item: syntax::Expr::Var(String::from("ltInt")),
                    },
                )],
            },
        };

        let instance_ord_int_result = tc
            .check_declaration(&mut HashMap::new(), &instance_ord_int_decl)
            .unwrap()
            .unwrap();
        tc.register_declaration(&instance_ord_int_result);

        let array_int_lt_decl = Spanned {
            pos: 0,
            item: syntax::Declaration::Definition {
                name: String::from("comparison"),
                ty: Type::Bool,
                args: Vec::new(),
                body: syntax::Expr::mk_app(
                    syntax::Expr::mk_app(
                        Spanned {
                            pos: 0,
                            item: syntax::Expr::Var(String::from("lt")),
                        },
                        Spanned {
                            pos: 0,
                            item: syntax::Expr::Array(vec![
                                Spanned {
                                    pos: 0,
                                    item: syntax::Expr::Int(0),
                                },
                                Spanned {
                                    pos: 0,
                                    item: syntax::Expr::Int(1),
                                },
                                Spanned {
                                    pos: 0,
                                    item: syntax::Expr::Int(2),
                                },
                            ]),
                        },
                    ),
                    Spanned {
                        pos: 0,
                        item: syntax::Expr::Array(vec![
                            Spanned {
                                pos: 0,
                                item: syntax::Expr::Int(4),
                            },
                            Spanned {
                                pos: 0,
                                item: syntax::Expr::Int(5),
                            },
                        ]),
                    },
                ),
            },
        };
        let eq_int_dict = core::Expr::mk_record(
            vec![(core::Expr::Int(0), core::Expr::Name(String::from("eqInt")))],
            None,
        );
        let eq_array_int_dict = core::Expr::mk_record(
            vec![(
                core::Expr::Int(0),
                core::Expr::mk_app(
                    core::Expr::Name(String::from("eqArray")),
                    core::Expr::mk_app(core::Expr::Name(String::from("eq")), eq_int_dict.clone()),
                ),
            )],
            None,
        );
        let ord_int_dict = core::Expr::mk_record(
            vec![
                (core::Expr::Int(0), eq_int_dict),
                (core::Expr::Int(1), core::Expr::Name(String::from("ltInt"))),
            ],
            None,
        );
        let lt_array_int = core::Expr::mk_app(
            core::Expr::Name(String::from("ltArray")),
            core::Expr::mk_app(core::Expr::Name(String::from("lt")), ord_int_dict),
        );
        let ord_array_int_dict = core::Expr::mk_record(
            vec![
                (core::Expr::Int(0), eq_array_int_dict),
                (core::Expr::Int(1), lt_array_int),
            ],
            None,
        );
        let expected_array_int_lt_result = Ok(Some(core::Declaration::Definition {
            name: String::from("comparison"),
            sig: TypeSig {
                ty_vars: Vec::new(),
                body: Type::Bool,
            },
            body: core::Expr::mk_app(
                core::Expr::mk_app(
                    core::Expr::mk_app(core::Expr::Name(String::from("lt")), ord_array_int_dict),
                    core::Expr::Array(vec![
                        core::Expr::Int(0),
                        core::Expr::Int(1),
                        core::Expr::Int(2),
                    ]),
                ),
                core::Expr::Array(vec![core::Expr::Int(4), core::Expr::Int(5)]),
            ),
        }));
        let actual_array_int_lt_result =
            tc.check_declaration(&mut HashMap::new(), &array_int_lt_decl);

        assert_eq!(
            expected_array_int_lt_result, actual_array_int_lt_result,
            "comparison = lt [0, 1, 2] [4, 5] is valid"
        )
    })
}

#[test]
fn unify_1() {
    crate::current_dir_with_tc!(|mut tc: Typechecker| {
        tc.bound_tyvars.insert(&[(Rc::from("r"), Kind::Row)]);
        let real = Type::mk_app(
            Type::mk_app(
                Type::Arrow,
                Type::mk_app(
                    Type::Record,
                    Type::mk_rowcons(Rc::from("x"), Type::Int, Type::Var(0)),
                ),
            ),
            Type::Int,
        );
        let m_0 = tc.fresh_typevar(Kind::Type);
        let m_1 = tc.fresh_typevar(Kind::Type);
        let holey = Type::mk_app(Type::mk_app(Type::Arrow, m_1), m_0);
        let expected = Ok(real.clone());
        let actual = {
            let context = UnifyTypeContext {
                expected: real.clone(),
                actual: holey.clone(),
            };
            tc.unify_type(&context, &real, &holey)
                .map(|_| tc.zonk_type(&holey))
        };
        assert_eq!(expected, actual)
    })
}
