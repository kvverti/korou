use std::num::IntErrorKind;

use crate::{span::{Spanned, Span}, tokens::Ident, token::{Token, TokenKind}, ast::Integer};

use super::{Parser, diagnostic::Diagnostics};

/// Holds parsing functions for atoms.
impl<'a> Parser<'a> {
    /// Parses an identifier from the next token.
    pub(super) fn ident(&mut self, ds: &mut Diagnostics) -> Option<Spanned<Ident>> {
        let t = self.expect(TokenKind::Ident, ds)?;
        let key = self.cache.intern(self.tz.src_for(Token::span(&t)));
        Some(Spanned::from_span_value(Span::from(self.tz.index()), Ident(key)))
    }

    /// Parses an integer from the next token.
    pub(super) fn integer(&mut self, ds: &mut Diagnostics) -> Option<Spanned<Integer>> {
        let t = self.tz.next();
        let (src, radix) = match *t {
            TokenKind::Number => (self.tz.src_for(Token::span(&t)), 10),
            TokenKind::BasePrefixNumber => {
                let src = self.tz.src_for(Token::span(&t));
                let (base, src) = (&src[1..2], &src[2..]);
                let radix = match base {
                    "x" | "X" => 16,
                    "c" | "C" => 8,
                    "b" | "B" => 2,
                    _ => panic!("This should probably be a parse error"),
                };
                (src, radix)
            }
            _ => {
                ds.error(format!("Expected this token: {:?}", TokenKind::Number));
                return None;
            }
        };
        let num = i64::from_str_radix(src, radix)
            .map_err(|err| match err.kind() {
                IntErrorKind::PosOverflow => ds.error("Integer too large"),
                _ => ds.error("Invalid base prefix"),
            })
            .ok()?;
        Some(Spanned::from_span_value(Span::from(self.tz.index()), Integer(num)))
    }
}

macro_rules! atom {
    (int $n:literal) => { $crate::ast::Integer($n) };
    ($id:ident) => { $crate::ast::Ident($id) };
    ($idx:literal : $($v:tt)*) => {
        $crate::span::Spanned::from_span_value(
            ($idx).into(),
            atom!($($v)*),
        )
    };
}
pub(crate) use atom;

#[cfg(test)]
mod tests {
    use crate::{cache::StringCache, tokenizer::Tokenizer};

    use super::*;
    
    #[test]
    fn atoms() {
        let mut cache = StringCache::new();
        let file_name = cache.intern("mysource.ku");
        let hello = cache.intern("hello");
        let tokenizer = Tokenizer::from_parts(file_name, "hello 17 0xc3f 0c19");
        let mut parser = Parser::from_parts(tokenizer, &mut cache);

        let mut diagnostics = Diagnostics::new();
        let expected = atom!(0: hello);
        let expected_diagnostics = Diagnostics::new();
        assert_eq!(expected, parser.ident(&mut diagnostics).expect("ident"));
        assert_eq!(expected_diagnostics, diagnostics);

        let mut diagnostics = Diagnostics::new();
        let expected = atom!(1: int 17);
        let expected_diagnostics = Diagnostics::new();
        assert_eq!(
            expected,
            parser.integer(&mut diagnostics).expect("integer1")
        );
        assert_eq!(expected_diagnostics, diagnostics);

        let mut diagnostics = Diagnostics::new();
        let expected = atom!(2: int 0xc3f);
        let expected_diagnostics = Diagnostics::new();
        assert_eq!(
            expected,
            parser.integer(&mut diagnostics).expect("integer2")
        );
        assert_eq!(expected_diagnostics, diagnostics);

        let mut diagnostics = Diagnostics::new();
        assert!(parser.integer(&mut diagnostics).is_none());
    }
}