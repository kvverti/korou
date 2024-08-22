use crate::{span::{Spanned, Span}, tokens::QualifiedIdent, token::TokenKind, diagnostic::Diagnostics};


impl<'a> Parser<'a> {
    /// Parses a qualified identifier from the next tokens.
    pub(super) fn qualified_ident(&mut self) -> Option<Spanned<QualifiedIdent>> {
        let mut paths = Vec::new();
        let (mut span, id) = self.ident()?.into_span_value();
        paths.push(id);
        while self.consume(TokenKind::Scope).is_some() {
            let (next_span, id) = self.ident()?.into_span_value();
            paths.push(id);
            Span::expand(&mut span, next_span);
        }
        Some(Spanned::from_span_value(span, QualifiedIdent(paths)))
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

use super::Parser;

#[cfg(test)]
mod tests {
    use crate::{cache::StringCache, tokenizer::Tokenizer, parse::declare_idents};

    use super::*;

    #[test]
    fn qualified_ident() {
        let mut cache = StringCache::new();
        let file_name = cache.intern("mysource.ku");
        declare_idents!(cache; foo bar baz);

        let tokenizer = Tokenizer::from_parts(file_name, "foo::bar::baz");
        let mut parser = Parser {
            tz: tokenizer,
            cache: &mut cache,
            ds: &mut Diagnostics::new(),
        };

        let expected = qident!(0..13: foo::bar::baz);
        assert_eq!(expected, parser.qualified_ident().unwrap());
        assert!(!parser.ds.has_errors());
    }
}
