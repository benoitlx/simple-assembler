mod lexer;
mod parser;

use lexer::Token;
use logos::Logos;
use miette::Result;
use colored::Colorize;

fn main() -> Result<()>{
    use std::io::Read;
    use std::env;

    let args: Vec<String> = env::args().collect();

    if let Ok(mut file) = std::fs::File::open(&args[1]) {
        let mut content = String::new();
        let _ = file.read_to_string(&mut content);

        let lex = Token::lexer(content.as_str());

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        let parser_output = parser::generate_bit_stream(&mut tokens, true, false, "\n");

        if !parser_output.2.is_empty() {
            for e in parser_output.2 {
                println!("{:?}", e.with_source_code(content.clone()));
            }
            panic!()
        }

        println!(
            "{}\n{}\n{}\n{}\n{}\n{}\n",
            "15 bits value".red(),
            "op/jump code".blue(),
            "mode bit".green(),
            "source A reg".yellow(),
            "source B reg".purple(),
            "dest reg".cyan()
        );

        println!("{}", parser_output.0);
    }

    Ok(())
}
