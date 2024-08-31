use super::{Effect, Ident, Statement, Type, TypedIdent, QualifiedIdent};
use crate::span::Span;

/// A concrete function.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Function {
    pub header: FunctionHeader,
    pub body: Vec<Statement>,
}

/// Function header, everything except the body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionHeader {
    pub name: Ident,
    pub type_params: Vec<Ident>,
    pub effect_params: Vec<Ident>,
    pub params: Vec<TypedIdent>,
    pub effects: Vec<Effect>,
    pub ret: Option<Vec<Type>>,
}

/// An item in the global or a namespace scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Function(Function),
    AbstractFunction(FunctionHeader),
    Finally {
        stmts: Vec<Statement>,
    },
    Effect {
        name: Ident,
        type_params: Vec<Ident>,
        effect_params: Vec<Ident>,
        body: Vec<Item>,
    },
    Import {
        module: QualifiedIdent,
    },
    Error {
        err_span: Span,
    },
}
