use crate::ast::{Expr, Type};

use super::Parser;

impl Parser<'_> {
    pub fn ty(&mut self) -> Type {
        // todo: remaining types
        // todo: type arguments
        let (span, ty_name) = self.qualified_ident().into_span_value();
        if let Expr::Ident(ty_name) = ty_name {
            Type::Named {
                name: ty_name,
                args: Vec::new(),
            }
        } else {
            Type::Error { err_span: span }
        }
    }
}
