use super::{Function, Statement, Type, Effect};
use crate::tokens::QualifiedIdent;

// effect foo {
//   fn bar(a: A) -> B;
// }
// straight from koka lol
// handler[A/e1] {
//   bar(a: A) { ..; resume(b); .. } -- type: (A, B -> R2/e) -> R2/e
//   (r: R) -> R2/e { .. }
//   finally { .. }
// }
// must be polymorphic over e if stored in a variable..
// todo: monomorphism restriction?

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectHandler {
    pub effect_name: QualifiedIdent,
    pub type_args: Vec<Type>,
    pub effect_args: Vec<Effect>,
    pub handlers: Vec<Function>,
    pub ret: Option<Function>,
    pub finally: Vec<Statement>,
}
