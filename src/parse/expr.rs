use crate::{ast::Expr, token::TokenKind, span::Spanned};

use super::{Parser, diagnostic::Diagnostics};

impl<'a> Parser<'a> {
    /// Parses an expression that can be the operand of a binary expression.
    /// This includes:
    /// - qualified identifiers: ident::...::ident
    /// - integer literals: 0xFF
    /// - parenthesized expressions: ( blockbased )
    pub fn unary_expr(&mut self, ds: &mut Diagnostics) -> Option<Expr> {
        let token = self.tz.peek();
        match *token {
            TokenKind::RoundL => {
                // parenthesized expression: (expr)
                self.advance();
                let expr = self.block_expr(ds);
                self.expect(TokenKind::RoundR, ds)?;
                expr
            }
            TokenKind::Ident => {
                // qualified identifier: ident::ident
                let (_, qid) = self.qualified_ident(ds)?.into_span_value();
                Some(Expr::Ident(qid))
            }
            TokenKind::Number | TokenKind::BasePrefixNumber => {
                // integer literal
                let int = self.integer(ds)?;
                Some(Expr::Int(*int))
            }
            _ => {
                ds.error(Spanned::span(&token), "Unexpected token");
                None
            }
        }
    }

    /// Parses a free binary expression. Free binary operators include:
    /// - member access: unary . ident
    /// - function call: unary ( args )
    /// - any unary expression: unary
    pub fn free_binary_expr(&mut self, ds: &mut Diagnostics) -> Option<Expr> {
        let mut expr = self.unary_expr(ds)?;
        while let Some(op_token) = self.consume_one_of(&[TokenKind::RoundL, TokenKind::Member]) {
            match *op_token {
                TokenKind::Member => {
                    let (_, rhs) = self.ident(ds)?.into_span_value();
                    expr = Expr::Member { recv: Box::new(expr), member: rhs }
                }
                TokenKind::RoundL => {
                    // todo: arguments
                    self.expect(TokenKind::RoundR, ds)?;
                    expr = Expr::Call { func: Box::new(expr), args: Vec::new() }
                }
                kind => unreachable!("Unknown free operator token {kind:?}"),
            }
        }
        Some(expr)
    }

    /// Parses a binary expression: a sequence of free binary expressions separated by the same operator.
    pub fn binary_expr(&mut self, ds: &mut Diagnostics) -> Option<Expr> {
        const OPERATOR_TOKENS: &[TokenKind] = &[
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
        ];
        let expr = self.free_binary_expr(ds);
        let Some(op_token) = self.consume_one_of(OPERATOR_TOKENS) else {
            return expr;
        };
        let rhs = self.free_binary_expr(ds)?;
        let mut operands = vec![expr?, rhs];
        while self.consume(*op_token).is_some() {
            operands.push(self.free_binary_expr(ds)?);
        }
        Some(Expr::Binary { op: (*op_token).try_into().expect("Operator token is not an operator"), operands })
    }

    /// Block-based expressions include:
    /// - if-then: if binary { block }
    /// - if-then-else: if binary { block } else { block }
    /// - do: do { block }
    /// - closure: { args -> block }
    /// - block-based function call: unary { args -> block }
    /// - any binary expression: binary
    pub fn block_expr(&mut self, ds: &mut Diagnostics) -> Option<Expr> {
        self.binary_expr(ds)
    }
}