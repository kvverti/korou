use crate::{
    ast::{Function, FunctionHeader, Ident, Item},
    parse::combinators,
    token::{Token, TokenKind},
};

use super::Parser;

impl Parser<'_> {
    /// A top-level or nested item.
    pub fn item(&mut self) -> Item {
        let head_tkn = self.tz.peek();
        match *head_tkn {
            TokenKind::Fn => self.function(),
            TokenKind::Finally => {
                self.advance();
                self.expect(TokenKind::CurlyL);
                let stmts = self.block_stmts();
                self.expect(TokenKind::CurlyR);
                Item::Finally { stmts }
            }
            TokenKind::Effect => {
                self.advance();
                let (_, name) = self.ident().into_span_value();
                let (type_params, effect_params) = self.generic_params();
                self.expect(TokenKind::CurlyL);
                let body = combinators::many(Self::item, &[TokenKind::CurlyR])(self);
                self.expect(TokenKind::CurlyR);
                Item::Effect {
                    name,
                    type_params,
                    effect_params,
                    body,
                }
            }
            _ => {
                self.advance();
                Item::Error {
                    err_span: Token::span(&head_tkn),
                }
            }
        }
    }

    /// Parses a function header. A function header must end in either ; or {
    /// fn name [ ident, ..., ident | ident, ..., ident ] ( nameandtype , ... , nameandtype ) / effect, ..., effect -> type
    pub fn function_header(&mut self) -> FunctionHeader {
        self.expect(TokenKind::Fn);
        let (_, name) = self.ident().into_span_value();
        let (type_params, effect_params) = self.generic_params();

        // parameters
        self.expect(TokenKind::RoundL);
        let params = combinators::comma_sequence(Self::name_and_type, &[TokenKind::RoundR])(self)
            .into_iter()
            .map(|v| v.into_span_value().1)
            .collect::<Option<Vec<_>>>()
            .unwrap_or_default();
        self.expect(TokenKind::RoundR);

        // effects
        let effects = if self.consume(TokenKind::Slash).is_some() {
            combinators::comma_sequence(Self::effect, &[TokenKind::Arrow])(self)
        } else {
            Vec::new()
        };

        self.expect(TokenKind::Arrow);
        let ret = self.fn_return_sequence();

        FunctionHeader {
            name,
            type_params,
            effect_params,
            params,
            effects,
            ret,
        }
    }

    /// Parses a function - a function header followed by either ; or = { block }.
    pub fn function(&mut self) -> Item {
        let header = self.function_header();
        if self.consume(TokenKind::Semi).is_some() {
            Item::AbstractFunction(header)
        } else {
            self.expect(TokenKind::Equals);
            self.expect(TokenKind::CurlyL);
            let body = self.block_stmts();
            self.expect(TokenKind::CurlyR);
            Item::Function(Function { header, body })
        }
    }

    /// Parses a list of type and effect parameters, including the delimiters.
    fn generic_params(&mut self) -> (Vec<Ident>, Vec<Ident>) {
        if self.consume(TokenKind::SquareL).is_some() {
            let type_params = combinators::comma_sequence(
                Self::ident,
                &[TokenKind::SquareR, TokenKind::Pipe],
            )(self)
            .into_iter()
            .map(|v| v.into_span_value().1)
            .collect::<Vec<_>>();
            let effect_params = if self.consume(TokenKind::Pipe).is_some() {
                combinators::comma_sequence(Self::ident, &[TokenKind::SquareR])(self)
                    .into_iter()
                    .map(|v| v.into_span_value().1)
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            self.expect(TokenKind::SquareR);
            (type_params, effect_params)
        } else {
            (Vec::new(), Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;

    #[test]
    fn valid_items_smoke() {
        let inputs = [
            "finally {}",
            "fn foo() ->;",
            "fn foo() -> ();",
            "fn foo() -> () ->;",
            "fn foo(x: A) -> (B) -> C;",
            "fn foo(x: () ->) -> {};",
            "fn foo[T, U | e]()/e -> (T, U);",
            "fn foo() -> = {}",
            "fn foo() -> {} = {}",
            "fn foo() -> () = {}",
        ];

        parse::tests::smoke_template(&inputs, |p| p.item());
    }
}
