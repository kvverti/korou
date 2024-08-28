use crate::{
    ast::{Conditional, Expr, TypedIdent},
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
    /// - keyword literals
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
            TokenKind::Return => {
                self.advance();
                Expr::Return
            }
            TokenKind::Ident => {
                // qualified identifier: ident::ident
                let (_, qid) = self.qualified_ident().into_span_value();
                Expr::Ident(qid)
            }
            TokenKind::Number | TokenKind::BasePrefixNumber => {
                // integer literal
                let (_, int) = self.integer().into_span_value();
                Expr::Int(int)
            }
            _ => {
                self.advance();
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
            .consume_one_of(&[TokenKind::RoundL, TokenKind::Dot])
            .into_span_value()
        {
            match op_token {
                TokenKind::Dot => {
                    let (_, rhs) = self.ident().into_span_value();
                    expr = Expr::Member {
                        recv: Box::new(expr),
                        member: rhs,
                    }
                }
                TokenKind::RoundL => {
                    let mut arguments_parser =
                        combinators::comma_sequence(Self::block_expr, &[TokenKind::RoundR]);
                    let args = arguments_parser(self);
                    self.expect(TokenKind::RoundR);
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
            let params =
                combinators::comma_sequence(Self::name_and_type, &[TokenKind::Arrow])(self)
                    .into_iter()
                    .map(|v| v.into_span_value().1)
                    .collect::<Option<Vec<_>>>()
                    .unwrap_or_default();
            self.expect(TokenKind::Arrow);
            params
        } else {
            Vec::new()
        };
        let stmts = self.block_stmts();
        Expr::Closure { params, stmts }
    }

    /// Block-based expressions include:
    /// - conditional: if binary { block } else ... else if binary { block } else { block }
    /// - do: do { block }
    /// - closure: { args -> block }
    /// - block-based function call: unary { args -> block }
    /// - handle expression: handle effect, ..., effect { function-or-finally }
    /// - any binary expression: binary
    pub fn block_expr(&mut self) -> Expr {
        let head_tkn = self.tz.peek();
        match *head_tkn {
            TokenKind::If => {
                // if-then or if-then-else
                self.advance();
                let mut cases = Vec::new();
                loop {
                    let condition = self.binary_expr();
                    self.expect(TokenKind::CurlyL);
                    let then_body = self.block_stmts();
                    self.expect(TokenKind::CurlyR);
                    cases.push(Conditional {
                        condition,
                        then_body,
                    });
                    if self.consume(TokenKind::Else).is_none() {
                        // no else block
                        break Expr::Conditional {
                            cases,
                            final_else: Vec::new(),
                        };
                    }
                    if self.consume(TokenKind::If).is_none() {
                        // final else block
                        self.expect(TokenKind::CurlyL);
                        let final_else = self.block_stmts();
                        self.expect(TokenKind::CurlyR);
                        break Expr::Conditional { cases, final_else };
                    }
                    // otherwise, continue with the next case
                }
            }
            TokenKind::Do => {
                // do expression (immediately invoked nullary closure)
                // or do-with expression (bind effect handler)
                self.advance();
                self.expect(TokenKind::CurlyL);
                let stmts = self.block_stmts();
                self.expect(TokenKind::CurlyR);
                if self.consume(TokenKind::With).is_none() {
                    // do-expression
                    return Expr::Do { stmts };
                }
                // do-with expression
                let handler = self.block_expr();
                Expr::DoWith {
                    stmts,
                    handler: Box::new(handler),
                }
            }
            TokenKind::CurlyL => {
                // closure
                self.advance();
                let closure = self.closure_body();
                self.expect(TokenKind::CurlyR);
                closure
            }
            TokenKind::Handle => {
                // handler
                self.advance();
                let impl_effects =
                    combinators::comma_sequence(Self::effect, &[TokenKind::CurlyL])(self);
                self.expect(TokenKind::CurlyL);
                let items = combinators::many(Self::item, &[TokenKind::CurlyR])(self);
                self.expect(TokenKind::CurlyR);
                Expr::Handler {
                    impl_effects,
                    items,
                }
            }
            _ => {
                // block function call or fallthrough
                let expr = self.binary_expr();
                if !matches!(expr, Expr::Binary { .. }) && self.consume(TokenKind::CurlyL).is_some()
                {
                    let block_arg = self.closure_body();
                    self.expect(TokenKind::CurlyR);
                    // determine whether we can add this argument to an existing function call
                    if let Expr::Call { func, mut args } = expr {
                        args.push(block_arg);
                        Expr::BlockCall { func, args }
                    } else {
                        Expr::BlockCall {
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
