use super::{Expr, TypedIdent};

/// Statements in a closure
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Statement {
    /// An expression statement
    Expr(Expr),
    /// A block expression statement
    BlockExpr(Expr),
    /// An expression that ends a block (i.e. without a trailing semicolon).
    BlockEndExpr(Expr),
    /// A let statement.
    Let {
        bindings: Vec<TypedIdent>,
        init: Expr,
    },
    /// An invocation of a continuation.
    Continue {
        cont: Expr,
        args: Vec<Expr>,
    },
}
