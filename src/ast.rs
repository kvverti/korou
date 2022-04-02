//! AST

mod effect;
mod expr;
mod func;
mod generic;
mod handler;
mod statement;
mod types;

pub use effect::Effect;
pub use expr::Expr;
pub use func::{Func, FuncHeader};
pub use generic::{Effects, TypeParam, TypedIdent};
pub use handler::EffectHandler;
pub use statement::Statement;
pub use types::{BaseType, Type};

#[cfg(test)]
mod tests {
    use crate::cache::StringCache;
    use crate::parser;
    use crate::tokens::{Ident, IntLiteral, IntRadix, Operator, QualifiedIdent};
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn integers() {
        let mut cache = StringCache::new();
        let inputs = ["34", "0xfe76c", "0c773", "0b11011"];
        let expected = [
            IntLiteral {
                value: 34,
                radix: IntRadix::Decimal,
            },
            IntLiteral {
                value: 0xfe76c,
                radix: IntRadix::Hex,
            },
            IntLiteral {
                value: 0o773,
                radix: IntRadix::Octal,
            },
            IntLiteral {
                value: 0b11011,
                radix: IntRadix::Binary,
            },
        ];
        let outputs = inputs
            .into_iter()
            .map(|s| {
                parser::IntParser::new()
                    .parse(&mut cache, s)
                    .expect("Failed to parse")
            })
            .collect::<Vec<_>>();
        assert_eq!(expected, outputs.as_slice());
    }

    #[test]
    fn expr() {
        let mut cache = StringCache::new();
        let input = "resume(r) + (x * f(foo.z, 0x4c7))";
        let expected = Expr::Call {
            func: Box::new(Expr::Op(Operator::Add)),
            args: vec![
                Expr::Resume(vec![Expr::Ident(QualifiedIdent::from(cache.intern("r")))]),
                Expr::Call {
                    func: Box::new(Expr::Op(Operator::Mul)),
                    args: vec![
                        Expr::Ident(QualifiedIdent::from(cache.intern("x"))),
                        Expr::Call {
                            func: Box::new(Expr::Ident(QualifiedIdent::from(cache.intern("f")))),
                            args: vec![
                                Expr::Member {
                                    recv: Box::new(Expr::Ident(QualifiedIdent::from(
                                        cache.intern("foo"),
                                    ))),
                                    member: Ident(cache.intern("z")),
                                },
                                Expr::Int(IntLiteral {
                                    value: 0x4c7,
                                    radix: IntRadix::Hex,
                                }),
                            ],
                        },
                    ],
                },
            ],
        };
        let output = parser::ExprParser::new()
            .parse(&mut cache, input)
            .expect("Failed to parse");
        assert_eq!(expected, output);
    }

    #[test]
    fn statements() {
        let mut cache = StringCache::new();
        let input = "{ let x: Int = f(y); if condition { return x; } else { x * x -> k; }; }";
        let expected = Expr::Closure {
            params: vec![],
            stmts: vec![
                Statement::Let {
                    bindings: vec![TypedIdent {
                        name: Ident(cache.intern("x")),
                        typ: Type::Base(BaseType::Simple(Ident(cache.intern("Int")))),
                    }],
                    init: Expr::Call {
                        func: Box::new(Expr::Ident(QualifiedIdent::from(cache.intern("f")))),
                        args: vec![Expr::Ident(QualifiedIdent::from(cache.intern("y")))],
                    },
                },
                Statement::Expr(Expr::IfElse {
                    condition: Box::new(Expr::Ident(QualifiedIdent::from(
                        cache.intern("condition"),
                    ))),
                    then_body: vec![Statement::Return(vec![Expr::Ident(QualifiedIdent::from(
                        cache.intern("x"),
                    ))])],
                    else_body: vec![Statement::Continue {
                        args: vec![Expr::Call {
                            func: Box::new(Expr::Op(Operator::Mul)),
                            args: vec![
                                Expr::Ident(QualifiedIdent::from(cache.intern("x"))),
                                Expr::Ident(QualifiedIdent::from(cache.intern("x"))),
                            ],
                        }],
                        cont: Expr::K,
                    }],
                }),
            ],
        };
        let output = parser::ExprParser::new()
            .parse(&mut cache, input)
            .expect("Failed to parse");
        assert_eq!(expected, output);
    }

    #[test]
    fn blocklike_statements() {
        let mut cache = StringCache::new();
        let input = "{ {}; if x {}; if y {} else {}; }";
        let empty = vec![Statement::Continue {
            args: vec![],
            cont: Expr::K,
        }];
        let expected = Expr::Closure {
            params: vec![],
            stmts: vec![
                Statement::Expr(Expr::Closure {
                    params: vec![],
                    stmts: empty.clone(),
                }),
                Statement::Expr(Expr::IfThen {
                    condition: Box::new(Expr::Ident(QualifiedIdent::from(cache.intern("x")))),
                    then_body: empty.clone(),
                }),
                Statement::Expr(Expr::IfElse {
                    condition: Box::new(Expr::Ident(QualifiedIdent::from(cache.intern("y")))),
                    then_body: empty.clone(),
                    else_body: empty,
                }),
            ],
        };
        let output = parser::ExprParser::new()
            .parse(&mut cache, input)
            .expect("Failed to parse");
        assert_eq!(expected, output);
    }

    #[test]
    fn function() {
        let mut cache = StringCache::new();
        let input = "fn foo[T, U | e e2](x: T, y: U)/e -> T/e2 { return x; }";
        let expected = Func {
            header: FuncHeader {
                name: Ident(cache.intern("foo")),
                type_params: vec![
                    TypeParam {
                        name: Ident(cache.intern("T")),
                    },
                    TypeParam {
                        name: Ident(cache.intern("U")),
                    },
                ],
                effect_params: vec![
                    TypeParam {
                        name: Ident(cache.intern("e")),
                    },
                    TypeParam {
                        name: Ident(cache.intern("e2")),
                    },
                ],
                params: vec![
                    TypedIdent {
                        name: Ident(cache.intern("x")),
                        typ: Type::Base(BaseType::Simple(Ident(cache.intern("T")))),
                    },
                    TypedIdent {
                        name: Ident(cache.intern("y")),
                        typ: Type::Base(BaseType::Simple(Ident(cache.intern("U")))),
                    },
                    TypedIdent {
                        name: Ident(cache.intern("k")),
                        typ: Type::Cont {
                            params: vec![Type::Base(BaseType::Simple(Ident(cache.intern("T"))))],
                            effects: Effects::from(vec![BaseType::Simple(Ident(
                                cache.intern("e2"),
                            ))]),
                        },
                    },
                ],
                effects: Effects::from(vec![
                    BaseType::Simple(Ident(cache.intern("e"))),
                    BaseType::Simple(Ident(cache.intern("e2"))),
                ]),
            },
            body: vec![Statement::Return(vec![Expr::Ident(QualifiedIdent::from(
                cache.intern("x"),
            ))])],
        };
        let output = parser::FuncParser::new()
            .parse(&mut cache, input)
            .expect("Failed to parse");
        assert_eq!(expected, output);
    }

    #[test]
    fn function2() {
        let mut cache = StringCache::new();
        let input = "fn foo(x: Bar)/e { return x; }";
        let expected = Func {
            header: FuncHeader {
                name: Ident(cache.intern("foo")),
                type_params: vec![],
                effect_params: vec![],
                params: vec![TypedIdent {
                    name: Ident(cache.intern("x")),
                    typ: Type::Base(BaseType::Simple(Ident(cache.intern("Bar")))),
                }],
                effects: Effects::from(vec![BaseType::Simple(Ident(cache.intern("e")))]),
            },
            body: vec![Statement::Return(vec![Expr::Ident(QualifiedIdent::from(
                cache.intern("x"),
            ))])],
        };
        let output = parser::FuncParser::new()
            .parse(&mut cache, input)
            .expect("Failed to parse");
        assert_eq!(expected, output);
    }

    #[test]
    fn zero_arg_function() {
        let mut cache = StringCache::new();
        let input = "fn foo() -> Foo {}";
        let expected = Func {
            header: FuncHeader {
                name: Ident(cache.intern("foo")),
                type_params: vec![],
                effect_params: vec![],
                params: vec![TypedIdent {
                    name: Ident(cache.intern("k")),
                    typ: Type::Cont {
                        params: vec![Type::Base(BaseType::Simple(Ident(cache.intern("Foo"))))],
                        effects: Effects(HashSet::new()),
                    },
                }],
                effects: Effects(HashSet::new()),
            },
            body: vec![Statement::Continue {
                args: vec![],
                cont: Expr::K,
            }],
        };
        let output = parser::FuncParser::new()
            .parse(&mut cache, input)
            .expect("Failed to parse");
        assert_eq!(expected, output);
    }

    #[test]
    fn zero_arg_continuation() {
        let mut cache = StringCache::new();
        let input = "fn foo() {}";
        let expected = Func {
            header: FuncHeader {
                name: Ident(cache.intern("foo")),
                type_params: vec![],
                effect_params: vec![],
                params: vec![],
                effects: Effects(HashSet::new()),
            },
            body: vec![Statement::Continue {
                args: vec![],
                cont: Expr::K,
            }],
        };
        let output = parser::FuncParser::new()
            .parse(&mut cache, input)
            .expect("Failed to parse");
        assert_eq!(expected, output);
    }

    #[test]
    fn typ() {
        let mut cache = StringCache::new();
        let input = "(Foo[T]/e ->, Bar) -> Baz/e2";
        let expected = Type::Cont {
            params: vec![
                Type::Cont {
                    params: vec![Type::Base(BaseType::Ctor {
                        name: Ident(cache.intern("Foo")),
                        args: vec![Type::Base(BaseType::Simple(Ident(cache.intern("T"))))],
                    })],
                    effects: Effects::from(vec![BaseType::Simple(Ident(cache.intern("e")))]),
                },
                Type::Base(BaseType::Simple(Ident(cache.intern("Bar")))),
                Type::Cont {
                    params: vec![Type::Base(BaseType::Simple(Ident(cache.intern("Baz"))))],
                    effects: Effects::from(vec![BaseType::Simple(Ident(cache.intern("e2")))]),
                },
            ],
            effects: Effects::from(vec![BaseType::Simple(Ident(cache.intern("e2")))]),
        };
        let output = parser::TypeParser::new()
            .parse(&mut cache, input)
            .expect("Failed to parse");
        assert_eq!(expected, output);
    }

    #[test]
    fn effect() {
        let mut cache = StringCache::new();
        let input = "effect foo[T] { fn bar(x: T) fn baz(y: T) -> T }";
        let expected = Effect {
            name: Ident(cache.intern("foo")),
            type_params: vec![TypeParam {
                name: Ident(cache.intern("T")),
            }],
            operators: vec![
                FuncHeader {
                    name: Ident(cache.intern("bar")),
                    type_params: vec![],
                    effect_params: vec![],
                    params: vec![TypedIdent {
                        name: Ident(cache.intern("x")),
                        typ: Type::Base(BaseType::Simple(Ident(cache.intern("T")))),
                    }],
                    effects: Effects(HashSet::new()),
                },
                FuncHeader {
                    name: Ident(cache.intern("baz")),
                    type_params: vec![],
                    effect_params: vec![],
                    params: vec![
                        TypedIdent {
                            name: Ident(cache.intern("y")),
                            typ: Type::Base(BaseType::Simple(Ident(cache.intern("T")))),
                        },
                        TypedIdent {
                            name: Ident(cache.intern("k")),
                            typ: Type::Cont {
                                params: vec![Type::Base(BaseType::Simple(Ident(
                                    cache.intern("T"),
                                )))],
                                effects: Effects(HashSet::new()),
                            },
                        },
                    ],
                    effects: Effects(HashSet::new()),
                },
            ],
        };
        let output = parser::EffectDefParser::new()
            .parse(&mut cache, input)
            .expect("Failed to parse");
        assert_eq!(expected, output);
    }

    #[test]
    fn handler() {
        let mut cache = StringCache::new();
        let input = "handle eff[Foo] { (x: T) -> T { x } fn bar(x: Foo) { } fn baz(x: Foo) -> Foo { resume(x); } finally {} }";
        let expected = EffectHandler {
            effect_name: QualifiedIdent::from(cache.intern("eff")),
            type_args: vec![Type::Base(BaseType::Simple(Ident(cache.intern("Foo"))))],
            effect_args: vec![],
            handlers: vec![
                Func {
                    header: FuncHeader {
                        name: Ident(cache.intern("bar")),
                        type_params: vec![],
                        effect_params: vec![],
                        params: vec![TypedIdent {
                            name: Ident(cache.intern("x")),
                            typ: Type::Base(BaseType::Simple(Ident(cache.intern("Foo")))),
                        }],
                        effects: Effects(HashSet::new()),
                    },
                    body: vec![Statement::Continue {
                        args: vec![],
                        cont: Expr::K,
                    }],
                },
                Func {
                    header: FuncHeader {
                        name: Ident(cache.intern("baz")),
                        type_params: vec![],
                        effect_params: vec![],
                        params: vec![
                            TypedIdent {
                                name: Ident(cache.intern("x")),
                                typ: Type::Base(BaseType::Simple(Ident(cache.intern("Foo")))),
                            },
                            TypedIdent {
                                name: Ident(cache.intern("k")),
                                typ: Type::Cont {
                                    params: vec![Type::Base(BaseType::Simple(Ident(
                                        cache.intern("Foo"),
                                    )))],
                                    effects: Effects(HashSet::new()),
                                },
                            },
                        ],
                        effects: Effects(HashSet::new()),
                    },
                    body: vec![Statement::Expr(Expr::Resume(vec![Expr::Ident(
                        QualifiedIdent::from(cache.intern("x")),
                    )]))],
                },
            ],
            ret: Some(Func {
                header: FuncHeader {
                    name: Ident(cache.intern("")),
                    type_params: vec![],
                    effect_params: vec![],
                    params: vec![
                        TypedIdent {
                            name: Ident(cache.intern("x")),
                            typ: Type::Base(BaseType::Simple(Ident(cache.intern("T")))),
                        },
                        TypedIdent {
                            name: Ident(cache.intern("k")),
                            typ: Type::Cont {
                                params: vec![Type::Base(BaseType::Simple(Ident(
                                    cache.intern("T"),
                                )))],
                                effects: Effects(HashSet::new()),
                            },
                        },
                    ],
                    effects: Effects(HashSet::new()),
                },
                body: vec![Statement::Continue {
                    args: vec![Expr::Ident(QualifiedIdent::from(cache.intern("x")))],
                    cont: Expr::K,
                }],
            }),
            finally: vec![],
        };
        let output = parser::HandlerParser::new()
            .parse(&mut cache, input)
            .expect("Failed to parse");
        assert_eq!(expected, output);
    }
}
