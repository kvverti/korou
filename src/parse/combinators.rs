use crate::token::TokenKind;

use super::Parser;

/// Collects a sequence of parsed values with no delineating token.
pub fn many<'a, 'b, T>(
    mut f: impl 'b + FnMut(&mut Parser<'a>) -> T,
    until: &'b [TokenKind],
) -> impl 'b + FnMut(&mut Parser<'a>) -> Vec<T> {
    move |this| {
        let mut values = Vec::new();
        while this.consume_one_of(until).is_none() {
            values.push(f(this));
        }
        values
    }
}

/// Collects a comma-delineated sequence of parsed values.
pub fn comma_sequence<'a, 'b, T>(
    mut f: impl 'b + FnMut(&mut Parser<'a>) -> T,
    until: &'b [TokenKind],
) -> impl 'b + FnMut(&mut Parser<'a>) -> Vec<T> {
    move |this| {
        let mut values = Vec::new();
        while this.consume_one_of(until).is_none() {
            values.push(f(this));
            if this.consume(TokenKind::Comma).is_none() {
                this.expect_one_of(until);
                break;
            }
        }
        values
    }
}
