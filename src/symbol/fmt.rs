use crate::symbol::SymbolTable;
use std::fmt::{self, Display, Formatter};

/// Trait for things that can be displayed with the context of a symbol table.
pub trait SymbolDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, symbol_table: &SymbolTable) -> fmt::Result;
}

pub struct SymbolFormatted<T>(pub T, pub SymbolTable);

impl<T: SymbolDisplay> Display for SymbolFormatted<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f, &self.1)
    }
}
