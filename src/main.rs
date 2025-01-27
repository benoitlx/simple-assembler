mod constants;
mod tokenizer;

use tokenizer::Token;
use logos::Logos;

fn main() {
    let lex = Token::lexer("A <- 256");

    for result in lex {
        match result {
            Ok(token) => println!("{:#?}", token),
            Err(_) => panic!("Err occured"),
        }
    }
}
