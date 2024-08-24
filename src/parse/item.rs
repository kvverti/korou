use crate::{
    ast::{Function, FunctionHeader, Item},
    parse::combinators,
    token::TokenKind,
};

use super::Parser;

impl Parser<'_> {
    /// Parses a function header. A function header must end in either ; or {
    /// fn name [ ident, ..., ident | ident, ..., ident ] ( nameandtype , ... , nameandtype ) / effect, ..., effect -> type
    pub fn function_header(&mut self) -> FunctionHeader {
        self.expect(TokenKind::Fn);
        let (_, name) = self.ident().into_span_value();

        let type_params;
        let effect_params;
        // type parameters
        if self.consume(TokenKind::SquareL).is_some() {
            type_params = combinators::comma_sequence(
                Self::ident,
                &[TokenKind::SquareR, TokenKind::Pipe],
            )(self)
            .into_iter()
            .map(|v| v.into_span_value().1)
            .collect::<Vec<_>>();
            if self.consume(TokenKind::Pipe).is_some() {
                effect_params =
                    combinators::comma_sequence(Self::ident, &[TokenKind::SquareR])(self)
                        .into_iter()
                        .map(|v| v.into_span_value().1)
                        .collect::<Vec<_>>();
            } else {
                effect_params = Vec::new();
            }
            self.expect(TokenKind::SquareR);
        } else {
            type_params = Vec::new();
            effect_params = Vec::new();
        }

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

        // return
        self.expect(TokenKind::Arrow);
        let ret =
            (!matches!(*self.tz.peek(), TokenKind::CurlyL | TokenKind::Semi)).then(|| self.ty());

        FunctionHeader {
            name,
            type_params,
            effect_params,
            params,
            effects,
            ret,
        }
    }

    /// Parses a function - a function header followed by a block.
    pub fn function(&mut self) -> Item {
        let header = self.function_header();
        self.expect(TokenKind::CurlyL);
        let body = self.block_stmts();
        self.expect(TokenKind::CurlyR);
        Item::Function(Function { header, body })
    }
}
