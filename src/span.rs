//! Contains the `Span` type which represents a range of source code.

use crate::cache::StringKey;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Span {
    pub file: StringKey,
    pub pos: usize,
    pub len: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}
