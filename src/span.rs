//! Contains the `Span` type which represents a range of source code.

use std::ops::{Deref, DerefMut};

use crate::cache::StringKey;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FileSpan {
    pub file: StringKey,
    pub pos: usize,
    pub len: usize,
}

impl FileSpan {
    /// Extends this span to cover the range of the given span.
    pub fn expand(this: &mut Self, other: Self) {
        assert_eq!(this.file, other.file);
        let pos = usize::min(this.pos, other.pos);
        let len = usize::max(this.pos + this.len, other.pos + other.len) - pos;
        this.pos = pos;
        this.len = len;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LocalSpan {
    pub offset: usize,
    pub len: usize,
}

impl LocalSpan {
    /// Extends this span to cover the range of the given span.
    pub fn expand(this: &mut Self, other: Self) {
        let offset = usize::min(this.offset, other.offset);
        let len = usize::max(this.offset + this.len, other.offset + other.len) - offset;
        this.offset = offset;
        this.len = len;
    }
}

/// A type representing a value that carries a local span.
pub type LocalSpanned<T> = Carrying<LocalSpan, T>;

/// A type representing a value that carries a file span.
pub type FileSpanned<T> = Carrying<FileSpan, T>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Carrying<C, T> {
    carry: C,
    value: T,
}

impl<C, T> Carrying<C, T> {
    /// Constructs a `Carrying<C, T>` from its component values.
    pub fn from_parts(carry: C, value: T) -> Self {
        Self { carry, value }
    }

    /// Returns the carry and value.
    pub fn into_parts(this: Self) -> (C, T) {
        (this.carry, this.value)
    }

    /// Retrieves the carry value.
    pub fn carry(this: Self) -> C {
        this.carry
    }

    pub fn map<R>(this: Self, f: impl FnOnce(T) -> R) -> Carrying<C, R> {
        Carrying {
            carry: this.carry,
            value: f(this.value),
        }
    }
}

impl<C, T> Deref for Carrying<C, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<C, T> DerefMut for Carrying<C, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
