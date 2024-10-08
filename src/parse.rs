//! The parser.

use crate::ast::Item;
use crate::cache::StringCache;
use crate::diagnostic::{Code, Diagnostics};
use crate::span::Spanned;
use crate::token::{Token, TokenKind};
use crate::tokenizer::Tokenizer;

mod atoms;
mod combinators;
mod expr;
mod item;
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

    /// Parse a single file. Might change later.
    pub fn file(&mut self) -> Vec<Item> {
        let mut items = Vec::new();
        while *self.tz.peek() != TokenKind::Eof {
            items.push(self.item());
        }
        items
    }
}

/// Shorthand for declaring many identifiers.
#[cfg(test)]
macro_rules! declare_idents {
    ($cache:ident; $($ids:ident)*) => {
        $(
            let $ids = $crate::ast::Ident::Ident($cache.intern(stringify!($ids)));
        )*
    };
}
#[cfg(test)]
pub(crate) use declare_idents;

#[cfg(test)]
mod tests {
    use crate::{
        cache::{StringCache, StringKey},
        diagnostic::Diagnostics,
        token::TokenKind,
        tokenizer::Tokenizer,
    };

    use super::Parser;

    pub fn smoke_template<R>(inputs: &[&str], mut parse_fn: impl FnMut(&mut Parser<'_>) -> R) {
        for input in inputs {
            let mut cache = StringCache::new();
            let mut ds = Diagnostics::new();
            let tz = Tokenizer::from_parts(StringKey::EMPTY, input);
            let mut parser = Parser {
                tz,
                cache: &mut cache,
                ds: &mut ds,
            };
            let _ = parse_fn(&mut parser);
            assert_eq!(TokenKind::Eof, *parser.tz.next(), "Failed to parse the entire input: {}", input);
            assert!(!parser.ds.has_errors(), "At input: {}", input);
        }
    }
}
