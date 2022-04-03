use crate::cache::StringKey;
use crate::span::Span;
use crate::token::Token;

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
    pub fn peek(&mut self) -> Option<Token> {
        self.consume_ws();
        rules::RULES
            .iter()
            .find_map(|&rule| rule(self.src))
            .map(|(kind, end)| Token {
                span: self.make_span(end),
                value: kind,
            })
    }

    /// Gets the next token and advances the tokenizer.
    pub fn next(&mut self) -> Option<Token> {
        let tkn = self.peek();
        if let Some(tkn) = tkn {
            self.src = &self.src[tkn.span.len..];
        }
        tkn
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
        assert_eq!(expected, tokenizer.next().expect("do"));

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 3,
                len: 3,
            },
            value: TokenKind::Ident,
        };
        assert_eq!(expected, tokenizer.next().expect("ident"));

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 7,
                len: 1,
            },
            value: TokenKind::Number,
        };
        assert_eq!(expected, tokenizer.next().expect("int1"));

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 9,
                len: 4,
            },
            value: TokenKind::Number,
        };
        assert_eq!(expected, tokenizer.next().expect("int2"));

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 14,
                len: 4,
            },
            value: TokenKind::Number,
        };
        assert_eq!(expected, tokenizer.next().expect("int3"));

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 18,
                len: 1,
            },
            value: TokenKind::Comma,
        };
        assert_eq!(expected, tokenizer.next().expect(", 1"));

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 19,
                len: 3,
            },
            value: TokenKind::Number,
        };
        assert_eq!(expected, tokenizer.next().expect("int4"));

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 22,
                len: 1,
            },
            value: TokenKind::Comma,
        };
        assert_eq!(expected, tokenizer.next().expect(", 2"));

        let expected = Token {
            span: Span {
                file: file_name,
                pos: 24,
                len: 1,
            },
            value: TokenKind::Number,
        };
        assert_eq!(expected, tokenizer.next().expect("int5"));
    }
}
