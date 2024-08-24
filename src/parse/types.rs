use crate::{
    ast::{Effect, Expr, Type},
    parse::combinators,
    token::TokenKind,
};

use super::Parser;

impl Parser<'_> {
    /// Parses a type name. Types are:
    /// Simple: qualident [ type, ..., type ]
    /// Continuation: ( type, ..., type ) / effect, effect -> type
    /// Closure: { type } / effect, effect
    pub fn ty(&mut self) -> Type {
        let head_tkn = self.tz.peek();
        match *head_tkn {
            TokenKind::CurlyL => {
                // closure type
                self.advance();
                let ret = self.ty();
                self.expect(TokenKind::CurlyR);
                let mut effects = Vec::new();
                if self.consume(TokenKind::Slash).is_some() {
                    effects.push(self.effect());
                    while self.consume(TokenKind::Comma).is_some() {
                        effects.push(self.effect());
                    }
                }
                Type::Closure {
                    ret: Box::new(ret),
                    effects,
                }
            }
            TokenKind::RoundL => {
                // continuation type
                self.advance();
                let args = combinators::comma_sequence(Self::ty, &[TokenKind::RoundR])(self);
                self.expect(TokenKind::RoundR);
                let effects = if self.consume(TokenKind::Slash).is_some() {
                    combinators::comma_sequence(Self::effect, &[TokenKind::Arrow])(self)
                } else {
                    Vec::new()
                };
                self.expect(TokenKind::Arrow);
                // check for a return type
                let ret = matches!(
                    *self.tz.peek(),
                    TokenKind::Ident | TokenKind::CurlyL | TokenKind::RoundL
                )
                .then(|| Box::new(self.ty()));
                Type::Continuation { args, ret, effects }
            }
            _ => {
                // simple type
                let (span, ty_name) = self.qualified_ident().into_span_value();
                let Expr::Ident(ty_name) = ty_name else {
                    return Type::Error { err_span: span };
                };
                let args = if self.consume(TokenKind::SquareL).is_some() {
                    let args = combinators::comma_sequence(Self::ty, &[TokenKind::SquareR])(self);
                    self.expect(TokenKind::SquareR);
                    args
                } else {
                    Vec::new()
                };
                Type::Simple {
                    name: ty_name,
                    args,
                }
            }
        }
    }

    /// Parses an effect name.
    pub fn effect(&mut self) -> Effect {
        // todo: fully design effects
        let (span, name) = self.qualified_ident().into_span_value();
        if let Expr::Ident(name) = name {
            Effect::Effect { name }
        } else {
            Effect::Error { err_span: span }
        }
    }
}
