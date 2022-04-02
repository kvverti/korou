use super::Effects;
use crate::tokens::Ident;

/// Base types. These are types which may be attached to effects.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum BaseType {
    Simple(Ident),
    Ctor { name: Ident, args: Vec<Type> },
}

/// Types. These may be the types of parameters to functions.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Type {
    Base(BaseType),
    /// A continuation accepts some number of parameters and performs some effects.
    Cont {
        params: Vec<Type>,
        effects: Effects,
    },
}
