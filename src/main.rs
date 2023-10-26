#![allow(dead_code)]

use std::{error::Error, io::{stdin, stdout, Write}};

use crate::{cache::StringCache, token::TokenKind, tokenizer::Tokenizer};

mod ast;
mod cache;
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
        let mut tz = Tokenizer::from_parts(file, &input);
        let tokens = std::iter::from_fn(|| Some(tz.next()))
            .take_while(|&el| *el != TokenKind::Eof)
            .collect::<Vec<_>>();
        println!("{:?}", tokens);
    }
    Ok(())
}

#[cfg(test)]
mod tests {}
