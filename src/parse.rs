//! The parser.

use crate::cache::StringCache;
use crate::parse::diagnostic::Diagnostics;
use crate::token::{Token, TokenKind};
use crate::tokenizer::Tokenizer;

mod atoms;
mod combinators;
pub mod diagnostic;
mod expr;
mod paths;

pub struct Parser<'a> {
    tz: Tokenizer<'a>,
    cache: &'a mut StringCache,
}

impl<'a> Parser<'a> {
    pub fn from_parts(tz: Tokenizer<'a>, cache: &'a mut StringCache) -> Self {
        Self { tz, cache }
    }

    /// Advances to the next token and asserts its kind.
    fn expect(&mut self, kind: TokenKind, ds: &mut Diagnostics) -> Option<Token> {
        self.expect_one_of(&[kind], ds)
    }

    /// Advances to the next token and asserts its kind.
    fn expect_one_of(&mut self, kind: &[TokenKind], ds: &mut Diagnostics) -> Option<Token> {
        self.tz
            .expect_one_of(kind)
            .map_err(|tkn| {
                ds.error(Token::span(&tkn), format!("Expected this token: `{:?}`", kind));
            })
            .ok()
    }

    /// Consumes the next token if it matches the given token kind.
    fn consume(&mut self, kind: TokenKind) -> Option<Token> {
        self.consume_one_of(&[kind])
    }

    /// Consumes the next token if it matches one of the given token kinds.
    fn consume_one_of(&mut self, kinds: &[TokenKind]) -> Option<Token> {
        let tkn = self.tz.peek();
        if kinds.contains(&tkn) {
            self.tz.next();
            Some(tkn)
        } else {
            None
        }
    }

    /// Advances the internal tokenizer, ignoring the next token.
    fn advance(&mut self) {
        self.tz.next();
    }
}

/// Shorthand for declaring many identifiers.
macro_rules! declare_idents {
    ($cache:ident; $($ids:ident)*) => {
        $(
            let $ids = $crate::ast::Ident($cache.intern(stringify!($ids)));
        )*
    };
}
pub(crate) use declare_idents;
