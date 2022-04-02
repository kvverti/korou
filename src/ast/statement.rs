use super::{Expr, TypedIdent};

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
