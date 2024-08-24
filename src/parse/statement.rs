use crate::{ast::Statement, token::TokenKind, parse::combinators};

use super::Parser;

impl Parser<'_> {
    /// Parses a statement. Statements include:
    /// - expression: blockexpr ;
    /// - block expression: blockexpr-end-with-{}
    /// - declaration: let ident: type, ..., ident: type = blockexpr ;
    /// - continuation: : freebinary binary , ... , binary ;
    pub fn stmt(&mut self) -> Statement {
        let head_tkn = self.tz.peek();
        match *head_tkn {
            TokenKind::Colon => {
                // continuation statement
                self.advance();
                let mut cont_args = combinators::comma_sequence(Self::binary_expr, &[TokenKind::Semi]);
                let cont = self.free_binary_expr();
                let args = cont_args(self);
                self.expect(TokenKind::Semi);
                Statement::Continue { cont, args }
            }
            TokenKind::Let => {
                // let statement
                self.advance();
                let bindings = combinators::comma_sequence(Self::name_and_type, &[TokenKind::Equals])(self)
                    .into_iter()
                    .map(|v| v.into_span_value().1)
                    .collect::<Option<Vec<_>>>()
                    .unwrap_or_default();
                self.expect(TokenKind::Equals);
                let init = self.block_expr();
                self.expect(TokenKind::Semi);
                Statement::Let { bindings, init }
            }
            _ => {
                // expression statement
                let expr = self.block_expr();
                if expr.is_block_expr() {
                    self.consume(TokenKind::Semi);
                    Statement::BlockExpr(expr)
                } else if self.consume(TokenKind::Semi).is_some() {
                    Statement::Expr(expr)
                } else {
                    Statement::BlockEndExpr(expr)
                }
            }
        }
    }

    pub fn block_stmts(&mut self) -> Vec<Statement> {
        combinators::many(Parser::stmt, &[TokenKind::CurlyR])(self)
    }
}