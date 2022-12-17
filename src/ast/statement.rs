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
    #[deprecated] // return is now a continuation
    Return(Vec<Expr>),
}
