use parsenip::lex::lex;
use parsenip::parse::parse;

use std::{
    error::Error,
    io::{self, Read},
};

fn main() -> Result<(), Box<dyn Error>> {
    // std in is raw html
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    dbg!("Input: {}", &buffer);
    let tokens = lex(&buffer)?;
    let root = parse(tokens)?;
    dbg!(root);
    Ok(())
}
