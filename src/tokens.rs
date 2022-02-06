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
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Ident(pub String);

impl From<&str> for Ident {
    fn from(v: &str) -> Self {
        Self(v.to_owned())
    }
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// Qualified identifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualifiedIdent(pub Vec<Ident>);

impl From<Ident> for QualifiedIdent {
    fn from(v: Ident) -> Self {
        Self(vec![v])
    }
}

impl From<&str> for QualifiedIdent {
    fn from(v: &str) -> Self {
        Self(vec![Ident::from(v)])
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
