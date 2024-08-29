use crate::{
    ast::{Effect, Type},
    parse::combinators,
    token::TokenKind,
};

use super::Parser;

impl Parser<'_> {
    /// Parses a type name. Types are:
    /// Simple: qualident [ type, ..., type ]
    /// Continuation: ( type, ..., type ) / effect, effect -> ... / effect, effect -> ( type, ..., type )
    /// Closure: { type, ..., type } / effect, effect
    pub fn ty(&mut self) -> Type {
        let head_tkn = self.tz.peek();
        match *head_tkn {
            TokenKind::CurlyL => {
                // closure type
                self.advance();
                let ret = combinators::comma_sequence(Self::ty, &[TokenKind::CurlyR])(self);
                self.expect(TokenKind::CurlyR);
                let mut effects = Vec::new();
                if self.consume(TokenKind::Slash).is_some() {
                    effects.push(self.effect());
                    while self.consume(TokenKind::Comma).is_some() {
                        effects.push(self.effect());
                    }
                }
                Type::Closure { ret, effects }
            }
            TokenKind::RoundL => {
                // continuation type
                let mut arg_lists = Vec::new();
                let ret_list;

                // the first argument list
                self.advance();
                let args = combinators::comma_sequence(Self::ty, &[TokenKind::RoundR])(self);
                self.expect(TokenKind::RoundR);
                let effects = if self.consume(TokenKind::Slash).is_some() {
                    combinators::comma_sequence(Self::effect, &[TokenKind::Arrow])(self)
                } else {
                    Vec::new()
                };
                self.expect(TokenKind::Arrow);
                arg_lists.push((args, effects));

                // parse any chained continuation types
                loop {
                    let next_tkn = *self.tz.peek();
                    if matches!(next_tkn, TokenKind::Ident | TokenKind::CurlyL) {
                        // single return type, no more to parse
                        ret_list = Some(vec![self.ty()]);
                        break;
                    }

                    if self.consume(TokenKind::RoundL).is_none() {
                        // no return type, no more to parse
                        ret_list = None;
                        break;
                    }

                    // multiple return types, possibly a single continuation return type - another argument list
                    let args = combinators::comma_sequence(Self::ty, &[TokenKind::RoundR])(self);
                    self.expect(TokenKind::RoundR);
                    if self.consume(TokenKind::Slash).is_some() {
                        // effects - must be followed by arrow
                        let effects =
                            combinators::comma_sequence(Self::effect, &[TokenKind::Arrow])(self);
                        self.expect(TokenKind::Arrow);
                        arg_lists.push((args, effects));
                    } else if self.consume(TokenKind::Arrow).is_some() {
                        // argument list for another continuation type
                        arg_lists.push((args, Vec::new()));
                    } else {
                        // return type list
                        ret_list = Some(args);
                        break;
                    }
                }

                // build the stack of continuations
                let (args, effects) = arg_lists
                    .pop()
                    .expect("Continuation type has at least one argument list");
                let mut ty = Type::Continuation {
                    args,
                    ret: ret_list,
                    effects,
                };
                while let Some((args, effects)) = arg_lists.pop() {
                    ty = Type::Continuation {
                        args,
                        ret: Some(vec![ty]),
                        effects,
                    };
                }
                ty
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
            "() -> ()",
            "() -> Foo[Bar]",
            "(Foo, Bar[Baz]) -> Quux",
            "()/ef ->",
            "{Foo}",
            "{Foo[Bar]}",
            "{}/ef",
            "()/ef, ef2, ef3 ->",
            "()/ef, ef2 -> ()/ef3 ->",
            "{() -> () -> {Foo, Bar}}",
            "() -> (Foo, Bar)",
            "() -> () ->",
            "() -> (() ->, Foo)",
        ];
        parse::tests::smoke_template(&inputs, |p| p.ty());
    }

    #[test]
    fn valid_effects_smoke() {
        // todo: instead of unit type, zero return values represented with ()
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
