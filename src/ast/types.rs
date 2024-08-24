use crate::{span::Span, tokens::Ident};

use super::QualifiedIdent;

/// Base types. These are types which may be attached to effects.
#[deprecated]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BaseType {
    Simple(Ident),
    Ctor { name: Ident, args: Vec<Type> },
}

/// Types. These may be the types of parameters to functions.
#[deprecated]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type0 {
    Base(BaseType),
    /// A continuation accepts some number of parameters and performs some effects.
    Cont {
        params: Vec<Type>,
        effects: Vec<BaseType>,
    },
}

/// A type in the AST.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    /// A named (possibly generic) type.
    Named {
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

#[derive(Clone, Debug, PartialEq, Eq)]
/// An effect in the AST.
pub struct Effect {
    pub name: QualifiedIdent,
}
