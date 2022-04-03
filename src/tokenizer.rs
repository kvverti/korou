use once_cell::sync::Lazy;
use regex::Regex;

use crate::cache::{StringCache, StringKey};
use crate::span::{Span, Spanned};
use crate::tokens::{Ident, IntLiteral, IntRadix};

static IDENT: Lazy<Regex> =
    Lazy::new(|| Regex::new("^[a-zA-Z_][a-zA-Z_0-9]*").expect("Ident regex"));
static BASE_PREFIX_INTEGER: Lazy<Regex> =
    Lazy::new(|| Regex::new("^0([xcbXCB])([0-9a-fA-F]+)").expect("Base prefix integer regex"));
static INTEGER: Lazy<Regex> = Lazy::new(|| Regex::new("^[1-9][0-9]*").expect("Integer regex"));

/// Lazy tokenizer.
#[derive(Debug)]
pub struct Tokenizer<'a> {
    cache: &'a mut StringCache,
    file_name: StringKey,
    base: &'a str,
    src: &'a str,
}

impl<'a> Tokenizer<'a> {
    /// Constructs a tokenizer from its component parts.
    pub fn from_parts(cache: &'a mut StringCache, file_name: StringKey, src: &'a str) -> Self {
        Self {
            cache,
            file_name,
            src,
            base: src,
        }
    }

    /// Eats white space.
    fn consume_ws(&mut self) {
        self.src = self.src.trim_start();
    }

    /// Advances the tokenizer by the given number of bytes and returns a span of those bytes.
    fn advance(&mut self, offset: usize) -> Span {
        let span = Span {
            file: self.file_name,
            pos: self.base.len() - self.src.len(),
            len: offset,
        };
        self.src = &self.src[offset..];
        span
    }

    /// Matches an exact sequence.
    pub fn expect(&mut self, exact_match: &str) -> Option<Span> {
        self.consume_ws();
        if self.src.starts_with(exact_match) {
            Some(self.advance(exact_match.len()))
        } else {
            None
        }
    }

    /// Matches a simple identifier.
    pub fn ident(&mut self) -> Option<Spanned<Ident>> {
        self.consume_ws();
        let matched = IDENT.find(self.src)?;
        Some(Spanned {
            span: self.advance(matched.end()),
            value: Ident(self.cache.intern(matched.as_str())),
        })
    }

    /// Matches an integer literal. This may not be a valid integer string representation.
    pub fn integer(&mut self) -> Option<Spanned<IntLiteral>> {
        self.consume_ws();
        if let Some(matched) = INTEGER.find(self.src) {
            Some(Spanned {
                span: self.advance(matched.end()),
                value: IntLiteral {
                    radix: IntRadix::Decimal,
                    value: matched.as_str().to_owned(),
                },
            })
        } else if let Some(captures) = BASE_PREFIX_INTEGER.captures(self.src) {
            let base_prefix = captures.get(1).unwrap();
            let integer = captures.get(2).unwrap();
            let radix = match base_prefix.as_str() {
                "x" | "X" => IntRadix::Hex,
                "c" | "C" => IntRadix::Octal,
                "b" | "B" => IntRadix::Binary,
                _ => panic!("Unexpected integer base prefix"),
            };
            Some(Spanned {
                span: self.advance(captures.get(0).unwrap().end()),
                value: IntLiteral {
                    radix,
                    value: integer.as_str().to_owned(),
                },
            })
        } else if self.src.starts_with("0") {
            Some(Spanned {
                span: self.advance(1),
                value: IntLiteral {
                    radix: IntRadix::Decimal,
                    value: "0".to_owned(),
                },
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes() {
        let mut cache = StringCache::new();
        let file_name = cache.intern("mysource.ku");
        let foo = cache.intern("foo");
        let mut tokenizer = Tokenizer::from_parts(&mut cache, file_name, "do foo 3 0xc3 0b01,0c9, 0");

        let expected = Span {
            file: file_name,
            pos: 0,
            len: 2,
        };
        assert_eq!(expected, tokenizer.expect("do").expect("do"));

        let expected = Spanned {
            span: Span {
                file: file_name,
                pos: 3,
                len: 3,
            },
            value: Ident(foo),
        };
        assert_eq!(expected, tokenizer.ident().expect("ident"));

        let expected = Spanned {
            span: Span {
                file: file_name,
                pos: 7,
                len: 1,
            },
            value: IntLiteral {
                radix: IntRadix::Decimal,
                value: "3".to_owned(),
            }
        };
        assert_eq!(expected, tokenizer.integer().expect("int1"));

        let expected = Spanned {
            span: Span {
                file: file_name,
                pos: 9,
                len: 4,
            },
            value: IntLiteral {
                radix: IntRadix::Hex,
                value: "c3".to_owned(),
            }
        };
        assert_eq!(expected, tokenizer.integer().expect("int2"));

        let expected = Spanned {
            span: Span {
                file: file_name,
                pos: 14,
                len: 4,
            },
            value: IntLiteral {
                radix: IntRadix::Binary,
                value: "01".to_owned(),
            }
        };
        assert_eq!(expected, tokenizer.integer().expect("int3"));

        let expected = Span {
            file: file_name,
            pos: 18,
            len: 1,
        };
        assert_eq!(expected, tokenizer.expect(",").expect(", 1"));

        let expected = Spanned {
            span: Span {
                file: file_name,
                pos: 19,
                len: 3,
            },
            value: IntLiteral {
                radix: IntRadix::Octal,
                value: "9".to_owned(),
            }
        };
        assert_eq!(expected, tokenizer.integer().expect("int4"));

        let expected = Span {
            file: file_name,
            pos: 22,
            len: 1,
        };
        assert_eq!(expected, tokenizer.expect(",").expect(", 2"));

        let expected = Spanned {
            span: Span {
                file: file_name,
                pos: 24,
                len: 1,
            },
            value: IntLiteral {
                radix: IntRadix::Decimal,
                value: "0".to_owned(),
            }
        };
        assert_eq!(expected, tokenizer.integer().expect("int5"));
    }
}
