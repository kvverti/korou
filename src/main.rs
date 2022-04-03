use lalrpop_util::lalrpop_mod;
use std::env::{self};
use std::error::Error;
use std::fs::File;
use std::io::Read;

mod ast;
mod block;
mod cache;
mod mir;
mod span;
mod symbol;
mod token;
mod tokenizer;
mod tokens;

lalrpop_mod!(parser);

fn main() -> Result<(), Box<dyn Error>> {
    let source_name = env::args().skip(1).next().unwrap(); // well toss you too, Option<T>: !Termination
    let mut source_file = File::open(source_name)?;
    let mut source = String::new();
    source_file.read_to_string(&mut source)?;
    Ok(())
}

#[cfg(test)]
mod tests {}
