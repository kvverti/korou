//! Parse errors.

use crate::span::FileSpan;
use crate::token::{Token, TokenKind};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub error: &'static str,
}
