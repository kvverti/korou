use crate::{
    ast::{Expr, TypedIdent},
    diagnostic::Code,
    span::Spanned,
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
                self.ds.add(Code::Unexpected, err_span, *token);
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
                        combinators::comma_sequence(Self::block_expr, &[TokenKind::RoundR]);
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

    /// Parses a name and type pair.
    pub fn name_and_type(&mut self) -> Spanned<Option<TypedIdent>> {
        let (name_span, name) = self.ident().into_span_value();
        let Some(name) = name else {
            return Spanned::from_span_value(name_span, None);
        };
        if self.expect(TokenKind::Colon).is_none() {
            return Spanned::from_span_value(name_span, None);
        }
        let ty = self.ty();
        let typed_ident = TypedIdent { name, ty };
        Spanned::from_span_value(name_span, Some(typed_ident))
    }

    /// A closure body: args -> stmts.
    pub fn closure_body(&mut self) -> Expr {
        let params = if *self.tz.peek2() == TokenKind::Colon {
            // parameters exist
            combinators::comma_sequence(Self::name_and_type, &[TokenKind::Arrow])(self)
                .into_iter()
                .map(|v| v.into_span_value().1)
                .collect::<Option<Vec<_>>>()
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let stmts = self.block_stmts();
        Expr::Closure { params, stmts }
    }

    /// Block-based expressions include:
    /// - if-then: if binary { block }
    /// - if-then-else: if binary { block } else { block }
    /// - do: do { block }
    /// - closure: { args -> block }
    /// - block-based function call: unary { args -> block }
    /// - any binary expression: binary
    pub fn block_expr(&mut self) -> Expr {
        let head_tkn = self.tz.peek();
        match *head_tkn {
            TokenKind::If => {
                // if-then or if-then-else
                self.advance();
                let condition = self.binary_expr();
                self.expect(TokenKind::CurlyL);
                let then_body = self.block_stmts();
                self.expect(TokenKind::CurlyR);
                match *self.consume(TokenKind::Else) {
                    Some(_) => {
                        self.expect(TokenKind::CurlyL);
                        let else_body = self.block_stmts();
                        self.expect(TokenKind::CurlyR);
                        Expr::IfElse {
                            condition: Box::new(condition),
                            then_body,
                            else_body,
                        }
                    }
                    None => Expr::IfThen {
                        condition: Box::new(condition),
                        then_body,
                    },
                }
            }
            TokenKind::Do => {
                // do expression (immediately invoked nullary closure)
                self.advance();
                self.expect(TokenKind::CurlyL);
                let stmts = self.block_stmts();
                self.expect(TokenKind::CurlyR);
                Expr::Do { stmts }
            }
            TokenKind::CurlyL => {
                // closure
                // todo: arguments
                self.advance();
                let closure = self.closure_body();
                self.expect(TokenKind::CurlyR);
                closure
            }
            _ => {
                // block function call or fallthrough
                let mut expr = self.binary_expr();
                if !matches!(expr, Expr::Binary { .. }) && self.consume(TokenKind::CurlyL).is_some()
                {
                    let block_arg = self.closure_body();
                    self.expect(TokenKind::CurlyR);
                    // determine whether we can add this argument to an existing function call
                    if let Expr::Call { ref mut args, .. } = expr {
                        args.push(block_arg);
                        expr
                    } else {
                        Expr::Call {
                            func: Box::new(expr),
                            args: vec![block_arg],
                        }
                    }
                } else {
                    expr
                }
            }
        }
    }
}
