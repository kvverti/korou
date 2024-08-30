use std::{
    error::Error,
    io::{stdin, stdout, Write},
};

use crate::{cache::StringCache, diagnostic::Diagnostics, parse::Parser, tokenizer::Tokenizer};

mod ast;
mod cache;
mod diagnostic;
mod mir;
mod parse;
mod span;
mod symbol;
mod token;
mod tokenizer;

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args_os().collect::<Vec<_>>();
    if let Some(filename) = args.get(1) {
        let file = std::fs::File::open(filename)?;
        let src = std::io::read_to_string(&file)?;
        let mut cache = StringCache::new();
        let filename = cache.intern(&filename.to_string_lossy());
        let tz = Tokenizer::from_parts(filename, &src);
        let mut ds = Diagnostics::new();
        let mut parser = Parser {
            tz,
            cache: &mut cache,
            ds: &mut ds,
        };
        let output = parser.file();
        for item in output {
            println!("Output: {item:?}\n");
        }
        println!("Diagnostics: {ds:?}");
    } else {
        // repl
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
            let mut parser = Parser {
                tz,
                cache: &mut cache,
                ds: &mut ds,
            };
            let output = parser.stmt();
            println!("Output: {:?}", output);
            println!("Diagnostics: {:?}", ds);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {}
