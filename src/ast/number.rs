//! Numbers.

/// Parsed integer literal.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Integer {
    Integer(i64),
    Error,
}
