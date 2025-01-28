mod constants;
mod tokenizer;

use logos::Logos;
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use thiserror::Error;
use tokenizer::Token;

#[derive(Error, Debug, Diagnostic)]
#[error("Unrecognized token")]
#[diagnostic(code(oops), url("https://rezoleo.fr"), help("Try with A *A V *V or C for a register"))]
pub struct TokenError {
    #[source_code]
    src: NamedSource<String>,

    #[label("problem here")]
    bad_bit: SourceSpan,
}

#[derive(Error, Diagnostic, Debug)]
pub enum TokenizerError {
    #[error(transparent)]
    #[diagnostic(code(tokernizer::io_error))]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    TokenError(#[from] TokenError),
}

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
            Err(_) => Err(TokenError {
                src: NamedSource::new(filename, contents.clone()),
                bad_bit: lex.span().into()
            })?
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let _ = tokenizer_app(env::args())?;

    Ok(())
}
