//! Identifiers and qualified identifiers.

use crate::cache::StringKey;

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
