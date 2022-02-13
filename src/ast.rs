use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::hash::{BuildHasher, Hash, Hasher};

use crate::tokens::{Ident, IntLiteral, Operator, QualifiedIdent};

/// Effect definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Effect {
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub operators: Vec<FuncHeader>,
}

// effect foo {
//   fn bar(a: A) -> B;
// }
// straight from koka lol
// handler[A/e1] {
//   bar(a: A) { ..; resume(b); .. } -- type: (A, B -> R2/e) -> R2/e
//   (r: R) -> R2/e { .. }
//   finally { .. }
// }
// must be polymorphic over e if stored in a variable..
// todo: monomorphism restriction?

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectHandler {
    pub type_args: Vec<Type>,
    pub effect_args: Vec<BaseType>,
    pub handlers: Vec<Func>,
    pub ret: Option<Func>,
    pub finally: Vec<Statement>,
}

/// Functions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Func {
    pub header: FuncHeader,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FuncHeader {
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub effect_params: Vec<TypeParam>,
    pub params: Vec<TypedIdent>,
    pub effects: Effects,
}

/// Type or effect parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeParam {
    pub name: Ident,
}

/// Function parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedIdent {
    pub name: Ident,
    pub typ: Type,
}

/// Expressions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    /// (Potentially) qualified identifier.
    Ident(QualifiedIdent),
    /// Simple integer literal.
    Int(IntLiteral),
    /// The implicit continuation, `k`.
    K,
    /// Explicit operator.
    Op(Operator),
    /// Member access.
    Member {
        recv: Box<Expr>,
        member: Ident,
    },
    /// Function call.
    Call {
        /// The function expression.
        func: Box<Expr>,
        /// The arguments to the function.
        args: Vec<Expr>,
    },
    /// Resumption
    Resume(Vec<Expr>),
    Closure {
        params: Vec<TypedIdent>,
        stmts: Vec<Statement>,
    },
    IfThen {
        condition: Box<Expr>,
        then_body: Vec<Statement>,
    },
    IfElse {
        condition: Box<Expr>,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    Handler(EffectHandler),
    DoWith {
        handler: Box<Expr>,
        stmts: Vec<Statement>,
    },
}

/// Statements in a closure
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Statement {
    Expr(Expr),
    Let {
        bindings: Vec<TypedIdent>,
        init: Expr,
    },
    Continue {
        args: Vec<Expr>,
        cont: Expr,
    },
    Return(Vec<Expr>),
}

/// Base types. These are types which may be attached to effects.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum BaseType {
    Simple(Ident),
    Ctor { name: Ident, args: Vec<Type> },
}

/// Types. These may be the types of parameters to functions.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Type {
    Base(BaseType),
    /// A continuation accepts some number of parameters and performs some effects.
    Cont {
        params: Vec<Type>,
        effects: Effects,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Effects(pub HashSet<BaseType>);

impl From<Vec<BaseType>> for Effects {
    fn from(v: Vec<BaseType>) -> Self {
        Self(v.into_iter().collect())
    }
}

impl Hash for Effects {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let build_hasher = RandomState::new();
        state.write_u64(
            self.0
                .iter()
                .map(|e| {
                    let mut hasher = build_hasher.build_hasher();
                    e.hash(&mut hasher);
                    hasher.finish()
                })
                .sum(),
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::StringCache;
    use crate::parser;
    use crate::tokens::IntRadix;

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
        let input = "handler[Foo] { (x: T) -> T { x } fn bar(x: Foo) { } fn baz(x: Foo) -> Foo { resume(x); } finally {} }";
        let expected = EffectHandler {
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
