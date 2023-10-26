use crate::token::TokenKind;

use super::{diagnostic::Diagnostics, Parser};

/// Collects a sequence of parsed values with no delineating token.
pub fn many<'a, 'b, T>(
    mut f: impl 'b + FnMut(&mut Parser<'a>, &mut Diagnostics) -> Option<T>,
    until: &'b [TokenKind],
) -> impl 'b + FnMut(&mut Parser<'a>, &mut Diagnostics) -> Option<Vec<T>> {
    move |this, ds| {
        let mut values = Vec::new();
        while this.consume_one_of(until).is_none() {
            values.push(f(this, ds)?);
        }
        Some(values)
    }
}

/// Collects a comma-delineated sequence of parsed values.
pub fn comma_sequence<'a, 'b, T>(
    mut f: impl 'b + FnMut(&mut Parser<'a>, &mut Diagnostics) -> Option<T>,
    until: &'b [TokenKind],
) -> impl 'b + FnMut(&mut Parser<'a>, &mut Diagnostics) -> Option<Vec<T>> {
    move |this, ds| {
        let mut values = Vec::new();
        while this.consume_one_of(until).is_none() {
            values.push(f(this, ds)?);
            if this.consume(TokenKind::Comma).is_none() {
                this.expect_one_of(until, ds)?;
                break;
            }
        }
        Some(values)
    }
}
