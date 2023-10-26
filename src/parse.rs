//! The parser.

use crate::ast::QualifiedIdent;
use crate::cache::StringCache;
use crate::parse::diagnostic::Diagnostics;
use crate::span::Spanned;
use crate::token::{Token, TokenKind};
use crate::tokenizer::Tokenizer;

mod atoms;
mod combinators;
mod diagnostic;

// Atoms

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
            .map_err(|_tkn| {
                ds.error(format!("Expected this token: {:?}", kind));
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

    /// Parses a qualified identifier from the next tokens.
    fn qualified_ident(&mut self, ds: &mut Diagnostics) -> Option<Spanned<QualifiedIdent>> {
        let mut paths = Vec::new();
        let id = *self.ident(ds)?;
        let bgn_idx = self.tz.index();
        paths.push(id);
        while self.consume(TokenKind::Scope).is_some() {
            let id = *self.ident(ds)?;
            paths.push(id);
        }
        let end_idx = self.tz.index();
        Some(Spanned::from_span_value((bgn_idx..end_idx + 1).into(), QualifiedIdent(paths)))
    }
}

macro_rules! qident {
    ($($components:ident)::*) => {
        $crate::ast::QualifiedIdent(vec![$($components),*])
    };
    ($bgn:literal .. $end:literal : $($ts:tt)*) => {
        $crate::span::Spanned::from_span_value(
            ($bgn..$end).into(),
            qident!($($ts)*),
        )
    }
}
pub(crate) use qident;

/// Shorthand for declaring many identifiers.
macro_rules! declare_idents {
    ($cache:ident; $($ids:ident)*) => {
        $(
            let $ids = $crate::ast::Ident($cache.intern(stringify!($ids)));
        )*
    };
}
pub(crate) use declare_idents;

#[cfg(test)]
mod tests {
    use crate::cache::StringCache;
    use crate::parse::diagnostic::Diagnostics;
    use crate::parse::Parser;
    use crate::tokenizer::Tokenizer;

    #[test]
    fn qualified_ident() {
        let mut cache = StringCache::new();
        let file_name = cache.intern("mysource.ku");
        declare_idents!(cache; foo bar baz);

        let tokenizer = Tokenizer::from_parts(file_name, "foo::bar::baz");
        let mut parser = Parser::from_parts(tokenizer, &mut cache);

        let mut diagnostics = Diagnostics::new();
        let expected = qident!(0..5: foo::bar::baz);
        let expected_diagnostics = Diagnostics::new();
        assert_eq!(expected, parser.qualified_ident(&mut diagnostics).unwrap());
        assert_eq!(expected_diagnostics, diagnostics);
    }
}
