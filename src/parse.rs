//! The parser.

use std::num::IntErrorKind;

use crate::ast::{Ident, Integer, QualifiedIdent};
use crate::cache::StringCache;
use crate::parse::diagnostic::Diagnostics;
use crate::parse::error::{Diagnostic, IntegerBase, ParseError};
use crate::span::Spanned;
use crate::token::{Token, TokenKind};
use crate::tokenizer::Tokenizer;

mod diagnostic;
mod error;

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
        self.tz
            .expect(kind)
            .map_err(|tkn| {
                ds.diagnostic(Diagnostic::Error(ParseError::UnexpectedToken {
                    tkn,
                    expected: kind,
                }));
            })
            .ok()
    }

    /// Peeks at the next token and tests its kind.
    fn peek(&mut self, kind: TokenKind) -> Option<Token> {
        let tkn = self.tz.peek();
        if tkn.value == kind {
            self.tz.next();
            Some(tkn)
        } else {
            None
        }
    }

    /// Parses an identifier from the next token.
    fn ident(&mut self, ds: &mut Diagnostics) -> Option<Spanned<Ident>> {
        let t = self.expect(TokenKind::Ident, ds)?;
        let key = self.cache.intern(self.tz.src_for(t.span));
        Some(Spanned {
            span: t.span,
            value: Ident(key),
        })
    }

    /// Parses an integer from the next token.
    fn integer(&mut self, ds: &mut Diagnostics) -> Option<Spanned<Integer>> {
        let t = self.tz.next();
        let (src, radix) = match t.value {
            TokenKind::Number => (self.tz.src_for(t.span), IntegerBase::Decimal),
            TokenKind::BasePrefixNumber => {
                let src = self.tz.src_for(t.span);
                let (base, src) = (&src[1..2], &src[2..]);
                let radix = match base {
                    "x" | "X" => IntegerBase::Hex,
                    "c" | "C" => IntegerBase::Octal,
                    "b" | "B" => IntegerBase::Binary,
                    _ => panic!("This should probably be a parse error"),
                };
                (src, radix)
            }
            _ => {
                ds.diagnostic(Diagnostic::Error(ParseError::UnexpectedToken {
                    tkn: t,
                    expected: TokenKind::Number,
                }));
                return None;
            }
        };
        let num = i64::from_str_radix(src, radix as u32)
            .map_err(|err| {
                ds.diagnostic(Diagnostic::Error(match err.kind() {
                    IntErrorKind::PosOverflow => ParseError::IntTooLarge { span: t.span },
                    _ => ParseError::InvalidIntDigit {
                        span: t.span,
                        base: radix,
                    },
                }))
            })
            .ok()?;
        Some(Spanned {
            span: t.span,
            value: Integer(num),
        })
    }

    /// Parses a qualified identifier from the next tokens.
    fn qualified_ident(&mut self, ds: &mut Diagnostics) -> Option<Spanned<QualifiedIdent>> {
        let mut paths = Vec::new();
        let Spanned {
            mut span,
            value: id,
        } = self.ident(ds)?;
        paths.push(id);
        while let Some(_) = self.peek(TokenKind::Scope) {
            let Spanned { span: s, value: id } = self.ident(ds)?;
            paths.push(id);
            span.expand(s);
        }
        Some(Spanned {
            span,
            value: QualifiedIdent(paths),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{Ident, Integer, QualifiedIdent};
    use crate::cache::StringCache;
    use crate::parse::diagnostic::Diagnostics;
    use crate::parse::error::{Diagnostic, IntegerBase, ParseError};
    use crate::parse::Parser;
    use crate::span::{Span, Spanned};
    use crate::tokenizer::Tokenizer;

    #[test]
    fn atoms() {
        let mut cache = StringCache::new();
        let file_name = cache.intern("mysource.ku");
        let hello = cache.intern("hello");
        let tokenizer = Tokenizer::from_parts(file_name, "hello 17 0xc3f 0c19");
        let mut parser = Parser::from_parts(tokenizer, &mut cache);

        let mut diagnostics = Diagnostics::new();
        let expected = Spanned {
            span: Span {
                file: file_name,
                pos: 0,
                len: 5,
            },
            value: Ident(hello),
        };
        let expected_diagnostics = Diagnostics::new();
        assert_eq!(expected, parser.ident(&mut diagnostics).expect("ident"));
        assert_eq!(expected_diagnostics, diagnostics);

        let mut diagnostics = Diagnostics::new();
        let expected = Spanned {
            span: Span {
                file: file_name,
                pos: 6,
                len: 2,
            },
            value: Integer(17),
        };
        let expected_diagnostics = Diagnostics::new();
        assert_eq!(
            expected,
            parser.integer(&mut diagnostics).expect("integer1")
        );
        assert_eq!(expected_diagnostics, diagnostics);

        let mut diagnostics = Diagnostics::new();
        let expected = Spanned {
            span: Span {
                file: file_name,
                pos: 9,
                len: 5,
            },
            value: Integer(0xc3f),
        };
        let expected_diagnostics = Diagnostics::new();
        assert_eq!(
            expected,
            parser.integer(&mut diagnostics).expect("integer2")
        );
        assert_eq!(expected_diagnostics, diagnostics);

        let mut diagnostics = Diagnostics::new();
        let expected_diagnostics =
            Diagnostics::from_diagnostics(vec![Diagnostic::Error(ParseError::InvalidIntDigit {
                span: Span {
                    file: file_name,
                    pos: 15,
                    len: 4,
                },
                base: IntegerBase::Octal,
            })]);
        assert!(parser.integer(&mut diagnostics).is_none());
        assert_eq!(expected_diagnostics, diagnostics);
    }

    #[test]
    fn qualified_ident() {
        let mut cache = StringCache::new();
        let file_name = cache.intern("mysource.ku");
        let foo_key = cache.intern("foo");
        let bar_key = cache.intern("bar");
        let baz_key = cache.intern("baz");
        let tokenizer = Tokenizer::from_parts(file_name, "foo::bar::baz");
        let mut parser = Parser::from_parts(tokenizer, &mut cache);

        let mut diagnostics = Diagnostics::new();
        let expected = Spanned {
            span: Span {
                file: file_name,
                pos: 0,
                len: 13,
            },
            value: QualifiedIdent(vec![Ident(foo_key), Ident(bar_key), Ident(baz_key)]),
        };
        let expected_diagnostics = Diagnostics::new();
        assert_eq!(expected, parser.qualified_ident(&mut diagnostics).unwrap());
        assert_eq!(expected_diagnostics, diagnostics);
    }
}
