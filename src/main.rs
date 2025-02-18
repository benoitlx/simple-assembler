mod lexer;
mod parser;

use lexer::Token;
use logos::Logos;

fn main() {
    use std::io::Read;

    if let Ok(mut file) = std::fs::File::open("tests/real_test/realistic_test.asm") {
        let mut content = String::new();
        let _ = file.read_to_string(&mut content);

        let lex = Token::lexer(content.as_str());

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        println!("{}", parser::generate_bit_stream_v2(&mut tokens));
    }
}
