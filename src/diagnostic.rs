//! Types relating to parser diagnostics.

use crate::span::Span;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DiagnosticKind {
    Error,
    Warn,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Code {
    Unexpected,
    IntegerTooLarge,
    InvalidIntegerDigit,
    InvalidIntegerBase,
}

impl Code {
    pub fn kind(&self) -> DiagnosticKind {
        use DiagnosticKind as K;
        match *self {
            Code::Unexpected => K::Error,
            Code::IntegerTooLarge => K::Error,
            Code::InvalidIntegerDigit => K::Error,
            Code::InvalidIntegerBase => K::Error,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: Code,
    pub span: Span,
    pub context: String,
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

    /// Adds a diagnostic with the given span and context.
    pub fn add(&mut self, code: Code, span: Span, context: impl ToString) {
        self.diagnostics.push(Diagnostic {
            code,
            span,
            context: context.to_string(),
        })
    }

    pub fn combine(&mut self, others: Self) {
        self.diagnostics.extend(others.diagnostics);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.code.kind() == DiagnosticKind::Error)
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
