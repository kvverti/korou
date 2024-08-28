use super::{Ident, Type};

/// Type or effect parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeParam {
    pub name: Ident,
}

/// Function parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedIdent {
    pub name: Ident,
    pub ty: Type,
}
