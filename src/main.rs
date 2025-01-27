mod constants;
mod tokenizer;

use tokenizer::Token;
use logos::Logos;
use std::fs::File;
use std::io::prelude::*;
use std::env;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut file = File::open(&args[1])?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;


    let lex = Token::lexer(contents.as_str());

    for result in lex {
        match result {
            Ok(token) => println!("{:#?}", token),
            Err(_) => panic!("Err occured"),
        }
    }

    Ok(())
}
