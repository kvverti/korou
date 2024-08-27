use crate::{
    span::{Span, Spanned},
    token::TokenKind,
    tokens::QualifiedIdent,
};

impl<'a> Parser<'a> {
    /// Parses a qualified identifier from the next tokens.
    pub(super) fn qualified_ident(&mut self) -> Spanned<QualifiedIdent> {
        let mut paths = Vec::new();
        let (mut span, id) = self.ident().into_span_value();
        paths.push(id);
        while self.consume(TokenKind::Scope).is_some() {
            let (next_span, id) = self.ident().into_span_value();
            paths.push(id);
            Span::expand(&mut span, next_span);
        }
        Spanned::from_span_value(span, QualifiedIdent(paths))
    }
}

#[cfg(test)]
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
#[cfg(test)]
pub(crate) use qident;

use super::Parser;

#[cfg(test)]
mod tests {
    use crate::{
        cache::StringCache, diagnostic::Diagnostics, parse::declare_idents, tokenizer::Tokenizer,
    };

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
        assert_eq!(expected, parser.qualified_ident());
        assert!(!parser.ds.has_errors());
    }
}
