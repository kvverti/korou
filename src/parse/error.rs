//! Parse errors.

use crate::span::Span;
use crate::token::{Token, TokenKind};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IntegerBase {
    Decimal = 10,
    Octal = 8,
    Binary = 2,
    Hex = 16,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    UnexpectedToken { tkn: Token, expected: TokenKind },
    IntTooLarge { span: Span },
    InvalidIntDigit { span: Span, base: IntegerBase },
}

impl ParseError {
    /// Gets the universal error code for this error. Parse errors use range 1.
    pub fn code(&self) -> u16 {
        match self {
            Self::UnexpectedToken { .. } => 1000,
            Self::IntTooLarge { .. } => 1001,
            Self::InvalidIntDigit { .. } => 1002,
        }
    }
}
