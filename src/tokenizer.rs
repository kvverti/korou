
use arraydeque::ArrayDeque;

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
    lookahead: ArrayDeque<Token, 2>,
}

impl<'a> Tokenizer<'a> {
    /// Constructs a tokenizer from its component parts.
    pub fn from_parts(file: StringKey, src: &'a str) -> Self {
        Self {
            file,
            src,
            base: src,
            lookahead: ArrayDeque::new(),
        }
    }

    /// Eats white space.
    fn consume_ws(&mut self) {
        self.src = self.src.trim_start();
    }

    fn next_token(&mut self) -> Token {
        self.consume_ws();
        let (kind, end) = rules::RULES
            .iter()
            .find_map(|&rule| rule(self.src))
            .unwrap_or_else(|| rules::unrecognized_char(self.src));
        let span = Span {
            pos: self.base.len() - self.src.len(),
            len: end,
        };
        self.src = &self.src[end..];
        Token::from_span_value(span, kind)
    }

    /// Gets the next token without advancing the tokenizer.
    pub fn peek(&mut self) -> Token {
        if let Some(tkn) = self.lookahead.front() {
            return *tkn;
        }
        let tkn = self.next_token();
        self.lookahead.push_back(tkn).expect("Lookahead has free capacity");
        tkn
    }

    /// Gets the second next token without advancing the tokenizer.
    pub fn peek2(&mut self) -> Token {
        while self.lookahead.len() < 2 {
            let tkn = self.next_token();
            self.lookahead.push_back(tkn).expect("Lookahead has free capacity");
        }
        self.lookahead[1]
    }

    /// Gets the next token and advances the tokenizer.
    pub fn next(&mut self) -> Token {
        self.lookahead.pop_front().unwrap_or_else(|| self.next_token())
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

        let expected = Token::from_span_value(Span { pos: 0, len: 2 }, TokenKind::Do);
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(Span { pos: 3, len: 3 }, TokenKind::Ident);
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(Span { pos: 7, len: 1 }, TokenKind::Number);
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(Span { pos: 9, len: 4 }, TokenKind::BasePrefixNumber);
        assert_eq!(expected, tokenizer.next());

        let expected =
            Token::from_span_value(Span { pos: 14, len: 4 }, TokenKind::BasePrefixNumber);
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(Span { pos: 18, len: 1 }, TokenKind::Comma);
        assert_eq!(expected, tokenizer.next());

        let expected =
            Token::from_span_value(Span { pos: 19, len: 3 }, TokenKind::BasePrefixNumber);
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(Span { pos: 22, len: 1 }, TokenKind::Comma);
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(Span { pos: 24, len: 1 }, TokenKind::Number);
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(Span { pos: 25, len: 0 }, TokenKind::Eof);
        assert_eq!(expected, tokenizer.next());
    }

    #[test]
    fn nuanced_tokenizes() {
        let mut cache = StringCache::new();
        let file_name = cache.intern("mysource.ku");
        let mut tokenizer = Tokenizer::from_parts(file_name, "  >= -> -\\>  ");

        let expected = Token::from_span_value(Span { pos: 2, len: 2 }, TokenKind::GtEquals);
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(Span { pos: 5, len: 2 }, TokenKind::Arrow);
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(Span { pos: 8, len: 1 }, TokenKind::Minus);
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(Span { pos: 9, len: 1 }, TokenKind::Unrecognized);
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(Span { pos: 10, len: 1 }, TokenKind::Gt);
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(Span { pos: 13, len: 0 }, TokenKind::Eof);
        assert_eq!(expected, tokenizer.next());
    }

    #[test]
    fn advances() {
        let mut cache = StringCache::new();
        let file_name = cache.intern("mysource.ku");
        let mut tokenizer = Tokenizer::from_parts(file_name, "foo bar");

        let expected = Token::from_span_value(Span { pos: 0, len: 3 }, TokenKind::Ident);
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(Span { pos: 4, len: 3 }, TokenKind::Ident);
        assert_eq!(expected, tokenizer.peek());
        assert_eq!(expected, tokenizer.peek());
        assert_eq!(expected, tokenizer.next());

        let expected = Token::from_span_value(Span { pos: 7, len: 0 }, TokenKind::Eof);
        assert_eq!(expected, tokenizer.peek());
        assert_eq!(expected, tokenizer.next());
        assert_eq!(expected, tokenizer.next());
    }

    #[test]
    fn advances2() {
        let mut cache = StringCache::new();
        let file_name = cache.intern("mysource.ku");
        let mut tokenizer = Tokenizer::from_parts(file_name, "foo bar");

        let expected1 = Token::from_span_value(Span { pos: 0, len: 3 }, TokenKind::Ident);
        let expected2 = Token::from_span_value(Span { pos: 4, len: 3 }, TokenKind::Ident);
        let expected3 = Token::from_span_value(Span { pos: 7, len: 0 }, TokenKind::Eof);

        assert_eq!(expected1, tokenizer.peek());
        assert_eq!(expected1, tokenizer.next());
        assert_eq!(expected3, tokenizer.peek2());
        assert_eq!(expected2, tokenizer.peek());
        assert_eq!(expected3, tokenizer.peek2());
        assert_eq!(expected3, tokenizer.peek2());
        assert_eq!(expected2, tokenizer.next());
        assert_eq!(expected3, tokenizer.peek());
        assert_eq!(expected3, tokenizer.next());
    }
}
