//! The parser.

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
    use crate::{cache::{StringCache, StringKey}, diagnostic::Diagnostics, tokenizer::Tokenizer, token::TokenKind};

    use super::Parser;

    #[test]
    fn valid_expressions_smoke() {
        let exprs = [
            "1",
            "a",
            "a.b",
            "a.b.c",
            "a::b",
            "a::b.c",
            "f()",
            "a.f()",
            "a::b.f()",
            "a.f().b",
            "f(1)",
            "f(1, 2)",
            "1 + 2",
            "1 + 2 + 3",
            "(1 + 2) * 3",
            "f(1 + 2)",
            "{}",
            "{ a }",
            "{ a; }",
            "{ a + b }",
            "{ {} }",
            "{ {}; }",
            "if 1 { 2 } else { 3 }",
            "do { 1 }",
            "{ x: Int, y: Int -> x + y }",
        ];

        for input in exprs {
            let mut cache = StringCache::new();
            let mut ds = Diagnostics::new();
            let tz = Tokenizer::from_parts(StringKey::EMPTY, input);
            let mut parser = Parser {
                tz,
                cache: &mut cache,
                ds: &mut ds,
            };
            let _ = parser.block_expr();
            assert_eq!(TokenKind::Eof, *parser.tz.next());
            assert!(!parser.ds.has_errors());
        }
    }
}
