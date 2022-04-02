use super::{FuncHeader, TypeParam};
use crate::symbol::{SymbolKey, SymbolTable};
use crate::tokens::Ident;

/// Effect definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Effect {
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub operators: Vec<FuncHeader>,
}

impl Effect {
    pub fn declare_symbols(&self, tbl: &mut SymbolTable, ctx: SymbolKey) -> Result<(), ()> {
        let effect_symb = tbl.define(self.name.0, ctx).ok_or(())?;
        let duplicate_type_params = self
            .type_params
            .iter()
            .filter_map(|t| {
                let s = tbl.define(t.name.0, effect_symb);
                if s.is_none() {
                    Some(t.name)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if duplicate_type_params.is_empty() {
            Ok(())
        } else {
            Err(())
        }
    }
}
