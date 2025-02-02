mod constants;
mod lexer;

use lexer::Token;
use logos::Logos;

fn main() {
    use std::io::Read;

    if let Ok(mut file) = std::fs::File::open("tests/test.asm") {
        let mut content = String::new();
        let _ = file.read_to_string(&mut content);

        let mut lex = Token::lexer(content.as_str());

        while let Some(result) = lex.next() {
            println!("{:?}", result);
        }
    }
}
