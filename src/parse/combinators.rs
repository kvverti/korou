use crate::token::TokenKind;

use super::Parser;

/// Collects a sequence of parsed values with no delineating token, until one of the given token kinds is reached.
/// EOF is automatically included as a stop token. The stop token is not consumed.
pub fn many<'a, 'b, T>(
    mut f: impl 'b + FnMut(&mut Parser<'a>) -> T,
    until: &'b [TokenKind],
) -> impl 'b + FnMut(&mut Parser<'a>) -> Vec<T> {
    move |this| {
        let mut values = Vec::new();
        loop {
            let next_token = this.tz.peek();
            if *next_token == TokenKind::Eof || until.contains(&next_token) {
                break;
            }
            values.push(f(this));
        }
        values
    }
}

/// Collects a comma-delineated sequence of parsed values. Stops when either a stop token is reached or a comma separator is not found.
/// The stop token is not consumed.
pub fn comma_sequence<'a, 'b, T>(
    mut f: impl 'b + FnMut(&mut Parser<'a>) -> T,
    until: &'b [TokenKind],
) -> impl 'b + FnMut(&mut Parser<'a>) -> Vec<T> {
    move |this| {
        let mut values = Vec::new();
        loop {
            let next_token = this.tz.peek();
            if until.contains(&next_token) {
                break;
            }
            values.push(f(this));
            if this.consume(TokenKind::Comma).is_none(){
                break;
            }
        }
        values
    }
}
