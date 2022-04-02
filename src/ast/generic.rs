use super::{BaseType, Type};
use crate::tokens::Ident;
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::hash::{BuildHasher, Hash, Hasher};

/// Type or effect parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeParam {
    pub name: Ident,
}

/// Function parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedIdent {
    pub name: Ident,
    pub typ: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Effects(pub HashSet<BaseType>);

impl From<Vec<BaseType>> for Effects {
    fn from(v: Vec<BaseType>) -> Self {
        Self(v.into_iter().collect())
    }
}

impl Hash for Effects {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let build_hasher = RandomState::new();
        state.write_u64(
            self.0
                .iter()
                .map(|e| {
                    let mut hasher = build_hasher.build_hasher();
                    e.hash(&mut hasher);
                    hasher.finish()
                })
                .sum(),
        );
    }
}
