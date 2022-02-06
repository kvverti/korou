use lalrpop_util::lalrpop_mod;
use std::env::{self};
use std::fs::File;
use std::io::Read;

mod ast;
mod symbol;
mod mir;
mod tokens;

lalrpop_mod!(parser);

fn main() -> Option<()> {
    let source_name = env::args().skip(1).next()?;
    let mut source_file = File::open(source_name).ok()?;
    let mut source = String::new();
    source_file.read_to_string(&mut source).ok()?;
    Some(())
}

#[cfg(test)]
mod tests {}
