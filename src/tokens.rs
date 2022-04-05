/// The integer literal, which may be written in different radixes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntLiteral {
    pub value: String,
    pub radix: IntRadix,
}

/// Integer radix.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IntRadix {
    Decimal,
    Octal,
    Hex,
    Binary,
}

pub use super::ast::{Ident, QualifiedIdent};

/// Binary operator.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}
