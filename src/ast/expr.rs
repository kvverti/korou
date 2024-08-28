use super::{Effect, Integer, Item, Statement, TypedIdent};
use crate::{
    span::Span,
    tokens::{Ident, Operator, QualifiedIdent},
};

/// A single case in an if-else ladder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conditional {
    pub condition: Expr,
    pub then_body: Vec<Statement>,
}

/// Expressions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    /// (Potentially) qualified identifier.
    Ident(QualifiedIdent),
    /// Simple integer literal.
    Int(Integer),
    /// Return literal.
    Return,
    /// The implicit continuation.
    Continue,
    /// A binary expression of a single operator
    Binary {
        op: Operator,
        operands: Vec<Expr>,
    },
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
    /// Block-based function call.
    BlockCall {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    Closure {
        params: Vec<TypedIdent>,
        stmts: Vec<Statement>,
    },
    Conditional {
        cases: Vec<Conditional>,
        /// may be empty
        final_else: Vec<Statement>,
    },
    Handler {
        impl_effects: Vec<Effect>,
        items: Vec<Item>,
    },
    Do {
        stmts: Vec<Statement>,
    },
    DoWith {
        stmts: Vec<Statement>,
        handler: Box<Expr>,
    },
    /// Error node.
    Error {
        err_span: Span,
    },
}

impl Expr {
    /// Determines whether this expression ends with a block.
    pub fn is_block_expr(&self) -> bool {
        match self {
            Self::BlockCall { .. }
            | Self::Conditional { .. }
            | Self::Closure { .. }
            | Self::Handler { .. }
            | Self::Do { .. } => true,
            Self::DoWith { handler, .. } => handler.is_block_expr(),
            _ => false,
        }
    }
}
