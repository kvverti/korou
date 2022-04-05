//! Specific rules for tokenizing subsets of tokens.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::token::TokenKind;

static IDENT: Lazy<Regex> =
    Lazy::new(|| Regex::new("^[a-zA-Z_][a-zA-Z_0-9]*").expect("Ident regex"));
static BASE_PREFIX_INTEGER: Lazy<Regex> =
    Lazy::new(|| Regex::new("^0[xcbXCB][0-9a-fA-F]+").expect("Base prefix integer regex"));
static INTEGER: Lazy<Regex> = Lazy::new(|| Regex::new("^[1-9][0-9]*").expect("Integer regex"));

/// All tokenizing rules. When more than one rule applies, the earliest rule is used.
pub const RULES: &[fn(&str) -> Option<(TokenKind, usize)>] = &[
    eof,
    width_two_punct,
    width_one_punct,
    base_prefix_number,
    unprefixed_number,
    keyword,
    ident,
];

pub fn width_two_punct(src: &str) -> Option<(TokenKind, usize)> {
    TokenKind::WIDTH_TWO_PUNCT
        .iter()
        .find(|p| src.starts_with(p.as_str()))
        .map(|p| (*p, p.as_str().len()))
}

pub fn width_one_punct(src: &str) -> Option<(TokenKind, usize)> {
    TokenKind::WIDTH_ONE_PUNCT
        .iter()
        .find(|p| src.starts_with(p.as_str()))
        .map(|p| (*p, p.as_str().len()))
}

pub fn keyword(src: &str) -> Option<(TokenKind, usize)> {
    TokenKind::KEYWORDS
        .iter()
        .find(|p| src.starts_with(p.as_str()))
        .map(|p| (*p, p.as_str().len()))
}

pub fn ident(src: &str) -> Option<(TokenKind, usize)> {
    IDENT.find(src).map(|m| (TokenKind::Ident, m.end()))
}

pub fn base_prefix_number(src: &str) -> Option<(TokenKind, usize)> {
    BASE_PREFIX_INTEGER
        .find(src)
        .map(|m| (TokenKind::BasePrefixNumber, m.end()))
}

pub fn unprefixed_number(src: &str) -> Option<(TokenKind, usize)> {
    INTEGER
        .find(src)
        .map(|m| m.end())
        .or_else(|| src.starts_with("0").then(|| 1))
        .map(|e| (TokenKind::Number, e))
}

pub fn eof(src: &str) -> Option<(TokenKind, usize)> {
    if src.is_empty() {
        Some((TokenKind::Eof, 0))
    } else {
        None
    }
}

pub fn unrecognized_char(src: &str) -> (TokenKind, usize) {
    let end = src.char_indices().nth(1).map(|t| t.0).unwrap_or(src.len());
    (TokenKind::Unrecognized, end)
}
