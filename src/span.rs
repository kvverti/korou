//! Contains the `Span` type which represents a range of source code.

use crate::cache::StringKey;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Span {
    pub file: StringKey,
    pub pos: usize,
    pub len: usize,
}

impl Span {
    /// Extends this span to cover the range of the given span.
    pub fn expand(&mut self, other: Self) {
        assert_eq!(self.file, other.file, "Files do not match");
        let pos = usize::min(self.pos, other.pos);
        let len = usize::max(self.pos + self.len, other.pos + other.len) - pos;
        self.pos = pos;
        self.len = len;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}
