// "bytecode w/ metadata" format
// - function / closure bodies are lists of operations
// - operations are typed
// - all symbols are already resolved
// - lists of operations are associated with a function/closure definition
//   - definitions have desugared signatures
//
// Literal format (for effect handlers and complex types):
// - I have no idea :(
//
// Closure format:
// - Closures can be suspended and resumed via their effect handlers
// - Note that `let x = foo();` actually creates a *new* closure
//
// Type format:
// - TBD
#![allow(dead_code)]

use crate::symbol::SymbolKey;

pub enum Opcode {
    // -> Any
    LoadValue(Value),
    // -> Any
    LoadLocal(usize),
    // Int, Int -> Int
    Add,
    // Int, Int -> Int
    Sub,
    // Int, Int -> Int
    Mul,
    // Record -> Record.x
    /// Member access.
    Access(usize),
    /// If-else branch with relative jumps
    Branch(i32, i32),
    // Args..., Cont ->
    Continue,
}

pub enum Value {
    /// Integer.
    Int(i64),
    /// Continuation/function/closure (they're all the same at this point).
    Cont(SymbolKey),
    /// Effect handler.
    Handler(/* oh lawdy */),
}

// |x, y| {
//   let z = x + y;
//   let w = foo(z);
//   w + 1
// }
// desugared syntax
// |x, y| {
//   let z = x + y;   -- simple ops, no continuation desugaring
//   foo(z, |w| { w + 1 -> k; });
// }
pub struct Closure {
    // includes parameters and simple `let`-bindings
    locals: Vec</* typed value */ ()>,
    code: Vec<Opcode>,
}

pub struct BoundClosure {
    /// The closure's bound effect handler. (make this an index)
    handler: Handler,
    closure: Closure,
}

// - Stores the effect handlers
// - Stores the CC (return)
// - Needs to store the finally
// - Needs to know about the finallies of the other bound handlers
// Handler stack? Handlers can reference the stack to get the finally...
// Closure lifetimes ensure that nothing important is reachable outside of its home stack frame.
pub struct Handler {
    // Takes the next finally and continues with it
    finally: Box<Closure>,
    /// The return continuation, which processes the return value before the handler drops.
    cc: Box<Closure>,
    /// Handles declared effect operations.
    actions: Vec<Closure>,
}

impl Handler {
    // need an empty "total" handler?
}
