use crate::cache::StringKey;
use crate::span::Span;
use crate::token::{Token, TokenKind};

mod rules;

/// Lazy tokenizer.
#[derive(Debug)]
pub struct Tokenizer<'a> {
    file_name: StringKey,
    base: &'a str,
    src: &'a str,
}

impl<'a> Tokenizer<'a> {
    /// Constructs a tokenizer from its component parts.
    pub fn from_parts(file_name: StringKey, src: &'a str) -> Self {
        Self {
            file_name,
            src,
            base: src,
        }
    }

    /// Eats white space.
    fn consume_ws(&mut self) {
        self.src = self.src.trim_start();
    }

    /// Constructs a span of the given number of bytes.
    fn make_span(&self, offset: usize) -> Span {
        Span {
            file: self.file_name,
            pos: self.base.len() - self.src.len(),
            len: offset,
        }
    }

    /// Gets the next token without advancing the tokenizer.
    pub fn peek(&mut self) -> Token {
        self.consume_ws();
        let (kind, end) = rules::RULES
            .iter()
            .find_map(|&rule| rule(self.src))
            .unwrap_or_else(|| rules::unrecognized_char(self.src));
        Token {
            span: self.make_span(end),
            value: kind,
        }
    }

    /// Gets the next token and advances the tokenizer.
    pub fn next(&mut self) -> Token {
        let tkn = self.peek();
        self.src = &self.src[tkn.span.len..];
        tkn
    }

    /// Gets the next token, advances the tokenizer, and tests the token's kind against the given kind.
    pub fn expect(&mut self, kind: TokenKind) -> Result<Token, Token> {
        let tkn = self.next();
        if tkn.value == kind {
            Ok(tkn)
        } else {
            Err(tkn)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::StringCache;
    use crate::token::TokenKind;

    use super::*;

    #[test]
    fn tokenizes() {
        let mut cache = StringCache::new();
        let file_name = cache.intern("mysource.ku");
        let mut tokenizer = Tokenizer::from_parts(file_name, "do foo 3 0xc3 0b01,0c9, 0");

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 0,
                len: 2,
            },
            value: TokenKind::Do,
        };
        assert_eq!(expected, tokenizer.next());

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 3,
                len: 3,
            },
            value: TokenKind::Ident,
        };
        assert_eq!(expected, tokenizer.next());

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 7,
                len: 1,
            },
            value: TokenKind::Number,
        };
        assert_eq!(expected, tokenizer.next());

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 9,
                len: 4,
            },
            value: TokenKind::Number,
        };
        assert_eq!(expected, tokenizer.next());

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 14,
                len: 4,
            },
            value: TokenKind::Number,
        };
        assert_eq!(expected, tokenizer.next());

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 18,
                len: 1,
            },
            value: TokenKind::Comma,
        };
        assert_eq!(expected, tokenizer.next());

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 19,
                len: 3,
            },
            value: TokenKind::Number,
        };
        assert_eq!(expected, tokenizer.next());

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 22,
                len: 1,
            },
            value: TokenKind::Comma,
        };
        assert_eq!(expected, tokenizer.next());

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 24,
                len: 1,
            },
            value: TokenKind::Number,
        };
        assert_eq!(expected, tokenizer.next());

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 25,
                len: 0,
            },
            value: TokenKind::Eof,
        };
        assert_eq!(expected, tokenizer.next());
    }

    #[test]
    fn nuanced_tokenizes() {
        let mut cache = StringCache::new();
        let file_name = cache.intern("mysource.ku");
        let mut tokenizer = Tokenizer::from_parts(file_name, "  >= -> -\\>  ");

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 2,
                len: 2,
            },
            value: TokenKind::GtEquals,
        };
        assert_eq!(expected, tokenizer.next());

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 5,
                len: 2,
            },
            value: TokenKind::Arrow,
        };
        assert_eq!(expected, tokenizer.next());

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 8,
                len: 1,
            },
            value: TokenKind::Minus,
        };
        assert_eq!(expected, tokenizer.next());

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 9,
                len: 1,
            },
            value: TokenKind::Unrecognized,
        };
        assert_eq!(expected, tokenizer.next());

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 10,
                len: 1,
            },
            value: TokenKind::Gt,
        };
        assert_eq!(expected, tokenizer.next());

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 13,
                len: 0,
            },
            value: TokenKind::Eof,
        };
        assert_eq!(expected, tokenizer.next());
    }
}
