//! Contains the `Span` type which represents a range of source code.

use std::{
    fmt::Display,
    ops::{Deref, DerefMut, Range},
};

/// A span of elements in some stream. Can be columns in source code or tokens.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Span {
    pub pos: usize,
    pub len: usize,
}

impl Span {
    /// Creates a span of no elements.
    pub const fn new() -> Self {
        Self { pos: 0, len: 0 }
    }

    /// Extends this span to cover the range of the given span.
    pub fn expand(this: &mut Self, other: Self) {
        let pos = usize::min(this.pos, other.pos);
        let len = usize::max(this.pos + this.len, other.pos + other.len) - pos;
        this.pos = pos;
        this.len = len;
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.pos, self.pos + self.len)
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Self {
            pos: value.start,
            len: value.end - value.start,
        }
    }
}

/// A value associated with a span. This can be used transparently where a value is required.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Spanned<T: ?Sized> {
    span: Span,
    value: T,
}

impl<T> Spanned<T> {
    pub fn from_span_value(span: Span, value: T) -> Self {
        Self { span, value }
    }

    pub fn into_span_value(self) -> (Span, T) {
        (self.span, self.value)
    }

    pub fn span(this: &Self) -> Span {
        this.span
    }
}

impl<T: ?Sized> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: ?Sized> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
