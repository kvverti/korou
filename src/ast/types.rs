use crate::span::Span;

use super::QualifiedIdent;

/// A type in the AST.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    /// A named (possibly generic) type.
    Simple {
        name: QualifiedIdent,
        args: Vec<Type>,
    },
    /// A continuation type.
    Continuation {
        args: Vec<Type>,
        ret: Option<Box<Type>>,
        effects: Vec<Effect>,
    },
    /// A closure type.
    Closure {
        ret: Box<Type>,
        effects: Vec<Effect>,
    },
    Error {
        err_span: Span,
    },
}

/// An effect in the AST.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effect {
    pub name: QualifiedIdent,
    pub args: Vec<Type>,
    pub meta_effects: Vec<Effect>,
}
