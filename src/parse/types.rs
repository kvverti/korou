use crate::{
    ast::{Expr, Type},
    parse::combinators,
    token::TokenKind,
};

use super::Parser;

impl Parser<'_> {
    /// Parses a type. Types are:
    /// Simple: qualident [ type, ..., type ]
    /// Continuation: ( type, ..., type ) -> type   (todo: effects)
    /// Closure: { type }   (todo: effects)
    pub fn ty(&mut self) -> Type {
        let head_tkn = self.tz.peek();
        match *head_tkn {
            TokenKind::CurlyL => {
                // closure type
                self.advance();
                let ret = self.ty();
                self.expect(TokenKind::CurlyR);
                Type::Closure {
                    ret: Box::new(ret),
                    effects: Vec::new(),
                }
            }
            TokenKind::RoundL => {
                // continuation type
                self.advance();
                let args = combinators::comma_sequence(Self::ty, &[TokenKind::RoundR])(self);
                self.expect(TokenKind::Arrow);
                // check for a return type
                let ret = matches!(
                    *self.tz.peek(),
                    TokenKind::Ident | TokenKind::CurlyL | TokenKind::RoundL
                )
                .then(|| Box::new(self.ty()));
                Type::Continuation {
                    args,
                    ret,
                    effects: Vec::new(),
                }
            }
            _ => {
                // simple type
                let (span, ty_name) = self.qualified_ident().into_span_value();
                let Expr::Ident(ty_name) = ty_name else {
                    return Type::Error { err_span: span };
                };
                let args = if self.consume(TokenKind::SquareL).is_some() {
                    combinators::comma_sequence(Self::ty, &[TokenKind::SquareR])(self)
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
}
