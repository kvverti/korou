use super::{EffectHandler, Integer, Statement, TypedIdent};
use crate::{
    span::Span,
    tokens::{Ident, Operator, QualifiedIdent},
};

/// Expressions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    /// (Potentially) qualified identifier.
    Ident(QualifiedIdent),
    /// Simple integer literal.
    Int(Integer),
    /// The implicit continuation, `k`.
    K,
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
    Do {
        stmts: Vec<Statement>,
    },
    DoWith {
        handler: Box<Expr>,
        stmts: Vec<Statement>,
    },
    /// Error node.
    Error {
        err_span: Span,
    },
}

impl Expr {
    pub fn is_block_expr(&self) -> bool {
        matches!(
            *self,
            Self::BlockCall { .. } | Self::IfThen { .. } | Self::IfElse { .. } | Self::Closure { .. }
        )
    }
}
