use super::{Effect, Statement, TypedIdent, Type};
use crate::{span::Span, tokens::Ident};

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
    pub ret: Option<Type>,
}

/// An item in the global or a namespace scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Function(Function),
    Finally {
        stmts: Vec<Statement>,
    },
    Error {
        err_span: Span,
    },
}
