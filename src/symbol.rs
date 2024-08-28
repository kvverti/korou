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
#![allow(dead_code)]
use crate::cache::StringKey;
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
    subpaths: HashMap<StringKey, SymbolKey>,
    /// The index in the symbol table this symbol's context symbol us stored at.
    /// The root context's context is itself.
    context: SymbolKey,
    /// The string represented by this symbol.
    string_key: StringKey,
}

impl Node {
    fn with_context(string_key: StringKey, context: SymbolKey) -> Self {
        Self {
            subpaths: HashMap::new(),
            context,
            string_key,
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
            nodes: vec![Node::with_context(StringKey::EMPTY, SymbolKey::ROOT)],
        }
    }

    /// Resolves a qualified symbol inside the given context.
    pub fn resolve(&self, name: &[StringKey], mut context: SymbolKey) -> Option<SymbolKey> {
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
    fn resolve_direct(&self, path_parts: &[StringKey], context: SymbolKey) -> Option<SymbolKey> {
        let mut search_root = &self.nodes[context.0];
        let mut key = None;
        for path in path_parts {
            key = search_root.subpaths.get(path).copied();
            search_root = &self.nodes[key?.0];
        }
        key
    }

    /// Defines a symbol under the given context. Returns the symbol key, or `None` if a
    /// symbol with the given name has already been defined.
    pub fn define(&mut self, name: StringKey, context: SymbolKey) -> Option<SymbolKey> {
        let next_index = self.nodes.len();
        let root = &mut self.nodes[context.0];
        if root.subpaths.contains_key(&name) {
            return None;
        }
        let key = SymbolKey(next_index);
        root.subpaths.insert(name, key);
        self.nodes.push(Node::with_context(name, context));
        Some(key)
    }

    /// Retrieves the string key for a given symbol. Multiple symbols may have the same string key.
    pub fn string_key(&self, key: SymbolKey) -> StringKey {
        self.nodes[key.0].string_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::StringCache;

    #[test]
    fn it_works() {
        let mut cache = StringCache::new();
        let ka = cache.intern("a");
        let kb = cache.intern("b");
        let kc = cache.intern("c");
        let mut symbol_table = SymbolTable::new();
        // symbols:
        //  1 - a
        //  2 - a::b
        //  3 - a::b::a
        //  4 - a::b::b
        //  5 - a::b::b::c
        //  6 - a::b::c
        let a = symbol_table.define(ka, SymbolKey::ROOT).expect("a");
        let a_b = symbol_table.define(kb, a).expect("a::b");
        let a_b_a = symbol_table.define(ka, a_b).expect("a::b::a");
        let a_b_b = symbol_table.define(kb, a_b).expect("a::b::b");
        let a_b_b_c = symbol_table.define(kc, a_b_b).expect("a::b::b::c");
        let a_b_c = symbol_table.define(kc, a_b).expect("a::b::c");
        assert!(symbol_table.define(kc, a_b).is_none());
        assert!(symbol_table.define(ka, SymbolKey::ROOT).is_none());

        assert_eq!(
            a,
            symbol_table.resolve(&[ka], SymbolKey::ROOT).expect("a/_"),
            "a/_"
        );
        assert_eq!(
            a_b,
            symbol_table
                .resolve(&[ka, kb], SymbolKey::ROOT)
                .expect("a::b/_"),
            "a::b/_"
        );
        assert_eq!(a_b, symbol_table.resolve(&[kb], a).expect("b/a"), "b/a");
        assert_eq!(
            a_b_b,
            symbol_table.resolve(&[kb], a_b).expect("b/a::b"),
            "b/a::b"
        );
        assert_eq!(
            a_b_c,
            symbol_table.resolve(&[kc], a_b).expect("c/a::b"),
            "c/a::b"
        );
        assert_eq!(a, symbol_table.resolve(&[ka], a).expect("a/a"), "a/a");
        assert_eq!(
            a_b_a,
            symbol_table.resolve(&[ka], a_b).expect("a/a::b"),
            "a/a::b"
        );
        assert_eq!(
            a_b_b_c,
            symbol_table.resolve(&[kb, kc], a_b).expect("b::c/a::b"),
            "b::c/a::b"
        );
        assert_eq!(
            a_b_a,
            symbol_table.resolve(&[kb, ka], a_b).expect("b::a/a::b"),
            "b::a/a::b"
        );
        assert!(
            symbol_table.resolve(&[kb], SymbolKey::ROOT).is_none(),
            "b/_"
        );
        assert!(symbol_table.resolve(&[kc], a).is_none(), "c/a")
    }
}
