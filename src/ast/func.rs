use super::{Statement, TypeParam, TypedIdent, BaseType};
use crate::tokens::Ident;

/// Functions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Func {
    pub header: FuncHeader,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FuncHeader {
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub effect_params: Vec<TypeParam>,
    pub params: Vec<TypedIdent>,
    pub effects: Vec<BaseType>,
}
