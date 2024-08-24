//! The parser.

use crate::cache::StringCache;
use crate::diagnostic::{Code, Diagnostics};
use crate::span::Spanned;
use crate::token::{Token, TokenKind};
use crate::tokenizer::Tokenizer;

mod atoms;
mod combinators;
mod expr;
mod paths;
mod statement;
mod types;

pub struct Parser<'a> {
    pub tz: Tokenizer<'a>,
    pub cache: &'a mut StringCache,
    pub ds: &'a mut Diagnostics,
}

impl<'a> Parser<'a> {
    /// Advances to the next token and asserts its kind.
    fn expect(&mut self, kind: TokenKind) -> Spanned<Option<TokenKind>> {
        self.expect_one_of(&[kind])
    }

    /// Advances to the next token and asserts its kind.
    fn expect_one_of(&mut self, kind: &[TokenKind]) -> Spanned<Option<TokenKind>> {
        match self.tz.expect_one_of(kind) {
            Ok(token) => Token::map(token, Some),
            Err(token) => {
                self.ds.add(Code::Unexpected, Token::span(&token), *token);
                Spanned::map(token, |_| None)
            }
        }
    }

    /// Consumes the next token if it matches the given token kind.
    fn consume(&mut self, kind: TokenKind) -> Spanned<Option<TokenKind>> {
        self.consume_one_of(&[kind])
    }

    /// Consumes the next token if it matches one of the given token kinds.
    fn consume_one_of(&mut self, kinds: &[TokenKind]) -> Spanned<Option<TokenKind>> {
        let tkn = self.tz.peek();
        if kinds.contains(&tkn) {
            self.tz.next();
            Token::map(tkn, Some)
        } else {
            Token::map(tkn, |_| None)
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
