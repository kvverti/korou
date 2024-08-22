use super::{EffectHandler, Statement, TypedIdent, Integer};
use crate::tokens::{Ident, Operator, QualifiedIdent};

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
    /// Resumption
    #[deprecated] // resume is now a function
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
