use crate::{
    ast::Expr,
    diagnostic::Code,
    token::{Token, TokenKind},
};

use super::{combinators, Parser};

impl<'a> Parser<'a> {
    /// Parses an expression that can be the operand of a binary expression.
    /// This includes:
    /// - qualified identifiers: ident::...::ident
    /// - integer literals: 0xFF
    /// - parenthesized expressions: ( blockbased )
    pub fn unary_expr(&mut self) -> Expr {
        let token = self.tz.peek();
        match *token {
            TokenKind::RoundL => {
                // parenthesized expression: (expr)
                self.advance();
                let expr = self.block_expr();
                self.expect(TokenKind::RoundR);
                expr
            }
            TokenKind::Ident => {
                // qualified identifier: ident::ident
                let (_, qid) = self.qualified_ident().into_span_value();
                qid
            }
            TokenKind::Number | TokenKind::BasePrefixNumber => {
                // integer literal
                let (span, int) = self.integer().into_span_value();
                int.map(Expr::Int).unwrap_or(Expr::Error { err_span: span })
            }
            _ => {
                let err_span = Token::span(&token);
                self.ds
                    .add(Code::Unexpected, err_span, format!("{:?}", *token));
                Expr::Error { err_span }
            }
        }
    }

    /// Parses a free binary expression. Free binary operators include:
    /// - member access: unary . ident
    /// - function call: unary ( args )
    /// - any unary expression: unary
    pub fn free_binary_expr(&mut self) -> Expr {
        let mut expr = self.unary_expr();
        while let (_, Some(op_token)) = self
            .consume_one_of(&[TokenKind::RoundL, TokenKind::Member])
            .into_span_value()
        {
            match op_token {
                TokenKind::Member => {
                    let (span, rhs) = self.ident().into_span_value();
                    expr = match rhs {
                        Some(rhs) => Expr::Member {
                            recv: Box::new(expr),
                            member: rhs,
                        },
                        None => Expr::Error { err_span: span },
                    }
                }
                TokenKind::RoundL => {
                    let mut arguments_parser =
                        combinators::comma_sequence(Self::binary_expr, &[TokenKind::RoundR]);
                    let args = arguments_parser(self);
                    expr = Expr::Call {
                        func: Box::new(expr),
                        args,
                    }
                }
                kind => unreachable!("Unknown free operator token {kind:?}"),
            }
        }
        expr
    }

    /// Parses a binary expression: a sequence of free binary expressions separated by the same operator.
    pub fn binary_expr(&mut self) -> Expr {
        const OPERATOR_TOKENS: &[TokenKind] = &[
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
        ];
        let expr = self.free_binary_expr();
        let (_, Some(op_token)) = self.consume_one_of(OPERATOR_TOKENS).into_span_value() else {
            return expr;
        };
        let rhs = self.free_binary_expr();
        let mut operands = vec![expr, rhs];
        while self.consume(op_token).is_some() {
            operands.push(self.free_binary_expr());
        }
        Expr::Binary {
            op: op_token
                .try_into()
                .expect("Operator token is not an operator"),
            operands,
        }
    }

    /// Block-based expressions include:
    /// - if-then: if binary { block }
    /// - if-then-else: if binary { block } else { block }
    /// - do: do { block }
    /// - closure: { args -> block }
    /// - block-based function call: unary { args -> block }
    /// - any binary expression: binary
    pub fn block_expr(&mut self) -> Expr {
        self.binary_expr()
    }
}
