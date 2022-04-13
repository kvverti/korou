//! Types relating to parser diagnostics.

use crate::parse::error::Diagnostic;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub const fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub(crate) fn from_diagnostics(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }

    pub fn diagnostic(&mut self, diag: Diagnostic) {
        self.diagnostics.push(diag);
    }

    pub fn combine(&mut self, others: Self) {
        self.diagnostics.extend(others.diagnostics);
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}
