/// The integer literal, which may be written in different radixes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntLiteral {
    pub value: String,
    pub radix: IntRadix,
}

/// Integer radix.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IntRadix {
    Decimal,
    Octal,
    Hex,
    Binary,
}

use crate::token::TokenKind;

pub use super::ast::{Ident, QualifiedIdent};

/// Binary operator.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnexpectedError;

impl TryFrom<TokenKind> for Operator {
    type Error = UnexpectedError;

    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::Plus => Ok(Self::Add),
            TokenKind::Minus => Ok(Self::Sub),
            TokenKind::Star => Ok(Self::Mul),
            TokenKind::Slash => Ok(Self::Div),
            _ => Err(UnexpectedError),
        }
    }
}
