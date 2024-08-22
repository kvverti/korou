#![allow(dead_code)]

use std::{error::Error, io::{stdin, stdout, Write}};

use crate::{parse::Parser, cache::StringCache, tokenizer::Tokenizer, diagnostic::Diagnostics};

mod ast;
mod cache;
mod diagnostic;
mod mir;
mod parse;
mod span;
mod symbol;
mod token;
mod tokenizer;
mod tokens;

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        print!("> ");
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;

        if input.trim() == ":quit" {
            break;
        }

        let mut cache = StringCache::new();
        let file = cache.intern("repl.ku");
        let tz = Tokenizer::from_parts(file, &input);
        let mut ds = Diagnostics::new();
        let mut cache = StringCache::new();
        let mut parser = Parser {
            tz,
            cache: &mut cache,
            ds: &mut ds,
        };
        let expr = parser.block_expr();
        println!("{:?}", expr);
    }
    Ok(())
}

#[cfg(test)]
mod tests {}
