// Symbol table
// - symbols have a unique key (numeric value)
// - symbols can be looked up by their path and context
//   - path is for namespaced symbols
//       "x" --> a,x  a,b,x
//   - context is for symbols declared within the scope defined by another symbol
//     - shadowing? Could be fine if we make `let` bindings act like closure bindings
//       let x = f(); --> f(|x| { ... })
//     - do closures get a symbol key? Are "context symbols" just scopes?
//       - "scopes" are just closures with no parameters, so closures *do* get symbol keys
//       - need to define a symbol for closures
//
// Need: resolve_symbol(symbol_name, context_symbol_id)
//       define_symbol(optional_symbol_name, context_symbol_id)

use std::collections::HashMap;

pub mod fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SymbolKey(usize);

impl SymbolKey {
    const ROOT: Self = Self(0);
}

/// Represents the root of a symbol (sub)tree.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Node {
    /// The symbols that have this symbol as its context.
    subpaths: HashMap<String, SymbolKey>,
    /// The index in the symbol table this symbol's context symbol us stored at.
    /// The root context's context is itself.
    context: SymbolKey,
}

impl Node {
    fn with_context(context: SymbolKey) -> Self {
        Self {
            subpaths: HashMap::new(),
            context,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolTable {
    // symbols are stored as a tree, root is index 0
    nodes: Vec<Node>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            nodes: vec![Node::with_context(SymbolKey::ROOT)],
        }
    }

    /// Resolves a qualified symbol inside the given context.
    pub fn resolve(&self, name: &[impl AsRef<str>], mut context: SymbolKey) -> Option<SymbolKey> {
        // b::c in a::b looks for
        // - a::b::b::c
        // - a::b::c
        // - b::c
        loop {
            match self.resolve_direct(name, context) {
                Some(key) => return Some(key),
                None => {
                    if context == SymbolKey::ROOT {
                        return None;
                    }
                    context = self.nodes[context.0].context
                }
            }
        }
    }

    /// Resolve a qualified symbol directly inside the given context.
    fn resolve_direct(
        &self,
        path_parts: &[impl AsRef<str>],
        context: SymbolKey,
    ) -> Option<SymbolKey> {
        let mut search_root = &self.nodes[context.0];
        let mut key = None;
        for path in path_parts {
            key = search_root.subpaths.get(path.as_ref()).copied();
            search_root = &self.nodes[key?.0];
        }
        key
    }

    /// Defines a symbol under the given context. Returns the symbol key, or `None` if a
    /// symbol with the given name has already been defined.
    pub fn define(&mut self, name: &str, context: SymbolKey) -> Option<SymbolKey> {
        let next_index = self.nodes.len();
        let root = &mut self.nodes[context.0];
        if root.subpaths.contains_key(name) {
            return None;
        }
        let key = SymbolKey(next_index);
        root.subpaths.insert(name.to_owned(), key);
        self.nodes.push(Node::with_context(context));
        Some(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut symbol_table = SymbolTable::new();
        // symbols:
        //  1 - a
        //  2 - a::b
        //  3 - a::b::a
        //  4 - a::b::b
        //  5 - a::b::b::c
        //  6 - a::b::c
        let a = symbol_table.define("a", SymbolKey::ROOT).expect("a");
        let a_b = symbol_table.define("b", a).expect("a::b");
        let a_b_a = symbol_table.define("a", a_b).expect("a::b::a");
        let a_b_b = symbol_table.define("b", a_b).expect("a::b::b");
        let a_b_b_c = symbol_table.define("c", a_b_b).expect("a::b::b::c");
        let a_b_c = symbol_table.define("c", a_b).expect("a::b::c");
        assert!(symbol_table.define("c", a_b).is_none());
        assert!(symbol_table.define("a", SymbolKey::ROOT).is_none());

        assert_eq!(
            a,
            symbol_table.resolve(&["a"], SymbolKey::ROOT).expect("a/_"),
            "a/_"
        );
        assert_eq!(
            a_b,
            symbol_table
                .resolve(&["a", "b"], SymbolKey::ROOT)
                .expect("a::b/_"),
            "a::b/_"
        );
        assert_eq!(a_b, symbol_table.resolve(&["b"], a).expect("b/a"), "b/a");
        assert_eq!(
            a_b_b,
            symbol_table.resolve(&["b"], a_b).expect("b/a::b"),
            "b/a::b"
        );
        assert_eq!(
            a_b_c,
            symbol_table.resolve(&["c"], a_b).expect("c/a::b"),
            "c/a::b"
        );
        assert_eq!(a, symbol_table.resolve(&["a"], a).expect("a/a"), "a/a");
        assert_eq!(
            a_b_a,
            symbol_table.resolve(&["a"], a_b).expect("a/a::b"),
            "a/a::b"
        );
        assert_eq!(
            a_b_b_c,
            symbol_table.resolve(&["b", "c"], a_b).expect("b::c/a::b"),
            "b::c/a::b"
        );
        assert_eq!(
            a_b_a,
            symbol_table.resolve(&["b", "a"], a_b).expect("b::a/a::b"),
            "b::a/a::b"
        );
        assert!(
            symbol_table.resolve(&["b"], SymbolKey::ROOT).is_none(),
            "b/_"
        );
        assert!(symbol_table.resolve(&["c"], a).is_none(), "c/a")
    }
}
