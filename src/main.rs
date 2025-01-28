mod constants;
mod error_handling;
mod tokenizer;

use error_handling::{TokenizerError, UnrecognizedToken};
use logos::Logos;
use miette::NamedSource;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use tokenizer::Token;

use miette::Result;
fn tokenizer_app(args: std::env::Args) -> Result<(), TokenizerError> {
    let args: Vec<String> = args.collect();

    let filename: &str = &args[1];

    let mut file = File::open(filename)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    let mut lex = Token::lexer(contents.as_str());

    while let Some(result) = lex.next() {
        match result {
            Ok(token) => println!("{:#?}", token),
            Err(_) => Err(UnrecognizedToken{
                src: NamedSource::new(filename, contents.clone()),
                src_span: lex.span().into(),
            })?,
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let _ = tokenizer_app(env::args())?;

    Ok(())
}
