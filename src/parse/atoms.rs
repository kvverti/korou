use std::num::IntErrorKind;

use crate::{ast::Integer, diagnostic::Code, span::Spanned, token::TokenKind, tokens::Ident};

use super::Parser;

// make a qualified ident an "atom" and use it to construct a unary expression

/// Holds parsing functions for atoms.
impl<'a> Parser<'a> {
    /// Parses an identifier from the next token.
    pub(super) fn ident(&mut self) -> Spanned<Option<Ident>> {
        let (span, kind) = self.expect(TokenKind::Ident).into_span_value();
        if kind.is_none() {
            Spanned::from_span_value(span, None)
        } else {
            let key = self.cache.intern(self.tz.src_for(span));
            Spanned::from_span_value(span, Some(Ident(key)))
        }
    }

    /// Parses an integer from the next token.
    pub(super) fn integer(&mut self) -> Spanned<Option<Integer>> {
        let (span, t) = self.tz.next().into_span_value();
        let (src, radix) = match t {
            TokenKind::Number => (self.tz.src_for(span), 10),
            TokenKind::BasePrefixNumber => {
                let src = self.tz.src_for(span);
                let (base, src) = src.split_at(2);
                let radix = match base {
                    "0x" | "0X" => 16,
                    "0c" | "0C" => 8,
                    "0b" | "0B" => 2,
                    _ => {
                        self.ds.add(Code::InvalidIntegerBase, span, base);
                        return Spanned::from_span_value(span, None);
                    }
                };
                (src, radix)
            }
            _ => {
                self.ds
                    .add(Code::Unexpected, span, TokenKind::Number);
                return Spanned::from_span_value(span, None);
            }
        };
        let num = i64::from_str_radix(src, radix)
            .map_err(|err| match err.kind() {
                IntErrorKind::PosOverflow => self.ds.add(Code::IntegerTooLarge, span, ""),
                IntErrorKind::InvalidDigit => self.ds.add(Code::InvalidIntegerDigit, span, ""),
                _ => unreachable!("Unexpected error: {:?}; on input: {} r {}", err, src, radix),
            })
            .ok();
        Spanned::from_span_value(span, num.map(Integer))
    }
}

macro_rules! atom {
    (int $n:literal) => { $crate::ast::Integer($n) };
    ($id:ident) => { $crate::ast::Ident($id) };
    ($bgn:literal .. $end:literal : $($v:tt)*) => {
        $crate::span::Spanned::from_span_value(
            ($bgn .. $end).into(),
            atom!($($v)*),
        )
    };
}
pub(crate) use atom;

#[cfg(test)]
mod tests {
    use crate::{cache::StringCache, diagnostic::Diagnostics, tokenizer::Tokenizer};

    use super::*;

    #[test]
    fn atoms() {
        let mut cache = StringCache::new();
        let file_name = cache.intern("mysource.ku");
        let hello = cache.intern("hello");
        let tokenizer = Tokenizer::from_parts(file_name, "hello 17 0xc3f 0c19");
        let mut parser = Parser {
            tz: tokenizer,
            cache: &mut cache,
            ds: &mut Diagnostics::new(),
        };

        let expected = atom!(0..5: hello);
        assert_eq!(
            expected,
            Spanned::map(parser.ident(), |v| v.expect("ident"))
        );
        assert!(!parser.ds.has_errors());

        let expected = atom!(6..8: int 17);
        assert_eq!(
            expected,
            Spanned::map(parser.integer(), |v| v.expect("integer1"))
        );
        assert!(!parser.ds.has_errors());

        let expected = atom!(9..14: int 0xc3f);
        assert_eq!(
            expected,
            Spanned::map(parser.integer(), |v| v.expect("integer2"))
        );
        assert!(!parser.ds.has_errors());

        assert!(parser.integer().is_none());
    }
}
