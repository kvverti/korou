use crate::cache::StringKey;
use crate::span::Span;
use crate::token::{Token, TokenKind};

mod rules;

/// Lazy tokenizer.
#[derive(Debug)]
pub struct Tokenizer<'a> {
    file: StringKey,
    base: &'a str,
    src: &'a str,
    /// Peeked token
    next: Option<Token>,
}

impl<'a> Tokenizer<'a> {
    /// Constructs a tokenizer from its component parts.
    pub fn from_parts(file: StringKey, src: &'a str) -> Self {
        Self {
            file,
            src,
            base: src,
            next: None,
        }
    }

    /// Eats white space.
    fn consume_ws(&mut self) {
        self.src = self.src.trim_start();
    }

    /// Constructs a span of the given number of bytes.
    fn make_span(&self, offset: usize) -> Span {
        Span {
            pos: self.base.len() - self.src.len(),
            len: offset,
        }
    }

    fn next_token(&mut self) -> Token {
        self.consume_ws();
        let (kind, end) = rules::RULES
            .iter()
            .find_map(|&rule| rule(self.src))
            .unwrap_or_else(|| rules::unrecognized_char(self.src));
        Token::from_span_value(self.make_span(end), kind)
    }

    /// Gets the next token without advancing the tokenizer.
    pub fn peek(&mut self) -> Token {
        if let Some(tkn) = self.next {
            return tkn;
        }
        let tkn = self.next_token();
        *self.next.insert(tkn)
    }

    /// Gets the next token and advances the tokenizer.
    pub fn next(&mut self) -> Token {
        let tkn = self.next.take().unwrap_or_else(|| self.next_token());
        self.src = &self.src[Token::span(&tkn).len..];
        tkn
    }

    /// Gets the next token, advances the tokenizer, and tests the token's kind against the given kind.
    pub fn expect_one_of(&mut self, kinds: &[TokenKind]) -> Result<Token, Token> {
        let tkn = self.next();
        if kinds.contains(&tkn) {
            Ok(tkn)
        } else {
            Err(tkn)
        }
    }

    /// Gets the source corresponding to the given span.
    /// # Panics
    /// This function panics if the span represents an invalid range.
    pub fn src_for(&self, span: Span) -> &str {
        &self.base[span.pos..span.pos + span.len]
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

        let expected = Token::from_span_value(
            Span {
                pos: 0,
                len: 2,
            },
            TokenKind::Do,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 3,
                len: 3,
            },
            TokenKind::Ident,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 7,
                len: 1,
            },
            TokenKind::Number,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 9,
                len: 4,
            },
            TokenKind::BasePrefixNumber,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 14,
                len: 4,
            },
            TokenKind::BasePrefixNumber,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 18,
                len: 1,
            },
            TokenKind::Comma,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 19,
                len: 3,
            },
            TokenKind::BasePrefixNumber,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 22,
                len: 1,
            },
            TokenKind::Comma,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 24,
                len: 1,
            },
            TokenKind::Number,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 25,
                len: 0,
            },
            TokenKind::Eof,
        );
        assert_eq!(expected, tokenizer.next());
    }

    #[test]
    fn nuanced_tokenizes() {
        let mut cache = StringCache::new();
        let file_name = cache.intern("mysource.ku");
        let mut tokenizer = Tokenizer::from_parts(file_name, "  >= -> -\\>  ");

        let expected = Token::from_span_value(
            Span {
                pos: 2,
                len: 2,
            },
            TokenKind::GtEquals,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 5,
                len: 2,
            },
            TokenKind::Arrow,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 8,
                len: 1,
            },
            TokenKind::Minus,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 9,
                len: 1,
            },
            TokenKind::Unrecognized,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 10,
                len: 1,
            },
            TokenKind::Gt,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 13,
                len: 0,
            },
            TokenKind::Eof,
        );
        assert_eq!(expected, tokenizer.next());
    }

    #[test]
    fn advances() {
        let mut cache = StringCache::new();
        let file_name = cache.intern("mysource.ku");
        let mut tokenizer = Tokenizer::from_parts(file_name, "foo bar");

        let expected = Token::from_span_value(
            Span {
                pos: 0,
                len: 3,
            },
            TokenKind::Ident,
        );
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 4,
                len: 3,
            },
            TokenKind::Ident,
        );
        assert_eq!(expected, tokenizer.peek());
        assert_eq!(expected, tokenizer.peek());
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(
            Span {
                pos: 7,
                len: 0,
            },
            TokenKind::Eof,
        );
        assert_eq!(expected, tokenizer.peek());
        assert_eq!(expected, tokenizer.next());
        assert_eq!(expected, tokenizer.next());
    }
}
