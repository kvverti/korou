//! Types relating to parser diagnostics.

use crate::span::Span;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DiagnosticType {
    Error,
    Warn,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub typ: DiagnosticType,
    pub span: Span,
    pub msg: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub const fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn error(&mut self, span: Span, msg: impl Into<String>) {
        self.diagnostics.push( Diagnostic { typ: DiagnosticType::Error, span, msg: msg.into() });
    }

    pub fn combine(&mut self, others: Self) {
        self.diagnostics.extend(others.diagnostics);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.typ == DiagnosticType::Error)
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}
