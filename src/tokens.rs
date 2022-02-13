use crate::cache::StringKey;

/// The integer literal, which may be written in different radixes.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct IntLiteral {
    pub value: i64,
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

/// Identifier.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Ident(pub StringKey);

/// Qualified identifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualifiedIdent(pub Vec<Ident>);

impl From<Ident> for QualifiedIdent {
    fn from(v: Ident) -> Self {
        Self(vec![v])
    }
}

impl From<StringKey> for QualifiedIdent {
    fn from(v: StringKey) -> Self {
        Self(vec![Ident(v)])
    }
}

/// Binary operator.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}
