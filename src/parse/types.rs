use crate::{
    ast::{Effect, Type},
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
            TokenKind::Unit => {
                self.advance();
                Type::Unit
            }
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
                    TokenKind::Unit | TokenKind::Ident | TokenKind::CurlyL | TokenKind::RoundL
                )
                .then(|| Box::new(self.ty()));
                Type::Continuation { args, ret, effects }
            }
            _ => {
                // simple type
                let (_, ty_name) = self.qualified_ident().into_span_value();
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

    /// Parses an effect.
    /// effect ident [ ty, ..., ty ]
    pub fn effect(&mut self) -> Effect {
        let mut effect_seq = Vec::new();
        loop {
            let (_, name) = self.qualified_ident().into_span_value();
            // generic arguments
            let args = if self.consume(TokenKind::SquareL).is_some() {
                let args = combinators::comma_sequence(Self::ty, &[TokenKind::SquareR])(self);
                self.expect(TokenKind::SquareR);
                args
            } else {
                Vec::new()
            };
            effect_seq.push(Effect {
                name,
                args,
                meta_effects: Vec::new(),
            });
            // note: we assume that an effect must begin with an ident token
            if *self.tz.peek() != TokenKind::Ident {
                break;
            }
        }
        let mut base_effect = effect_seq
            .pop()
            .expect("Effect loop is always run at least once.");
        base_effect.meta_effects = effect_seq;
        base_effect
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;

    #[test]
    fn valid_types_smoke() {
        let inputs = [
            "Foo",
            "Foo[Bar]",
            "Foo[Bar, Baz]",
            "() ->",
            "() -> Unit",
            "() -> Foo[Bar]",
            "(Foo, Bar[Baz]) -> Quux",
            "()/ef ->",
            "{Foo}",
            "{Foo[Bar]}",
            "{Unit}/ef",
            "()/ef, ef2, ef3 ->",
            "()/ef, ef2 -> ()/ef3 ->",
            "{() -> () -> {Unit}}",
        ];
        parse::tests::smoke_template(&inputs, |p| p.ty());
    }

    #[test]
    fn valid_effects_smoke() {
        let inputs = [
            "foo",
            "foo[Bar]",
            "foo[()/bar ->]",
            "foo bar",
            "foo[A] bar[B]",
            "foo bar baz",
        ];
        parse::tests::smoke_template(&inputs, |p| p.effect());
    }
}
