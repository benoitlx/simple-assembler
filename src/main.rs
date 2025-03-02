mod lexer;
mod parser;

use clap::Parser;
use colored::Colorize;
use lexer::Token;
use logos::Logos;

/// Simple cli to parse and generate bit stream for my custom assembly language
#[derive(Parser)]
struct Cli {
    /// assembly file path
    file_path: String,

    /// whether to colorize the bit stream output
    #[arg(short = 'c', long = "color")]
    color: bool,

    /// whether to print debug messages
    #[arg(short = 'd', long = "debug")]
    debug: bool,

    /// separator between each words in the bit stream
    #[arg(short = 's', long = "sep", default_value_t = String::from(""))]
    sep: String,
}

fn main() {
    use std::io::Read;

    let args = Cli::parse();

    if let Ok(mut file) = std::fs::File::open(args.file_path.clone()) {
        let mut content = String::new();
        let _ = file.read_to_string(&mut content);

        let lex = Token::lexer(content.as_str());

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        let (bit_stream, _, errors) =
            parser::generate_bit_stream(&mut tokens, args.color, args.debug, &args.sep);

        let error_number = errors.len();
        if error_number > 0 {
            for e in errors {
                println!("{:?}", e.with_source_code(content.clone()));
            }

            println!("{} errors found in {}", error_number, args.file_path);
            return;
        }

        if args.debug {
            println!(
                "{}\n{}\n{}\n{}\n{}\n{}\n",
                "15 bits value".red(),
                "op/jump code".blue(),
                "mode bit".green(),
                "source A reg".yellow(),
                "source B reg".purple(),
                "dest reg".cyan()
            );
        }

        println!("{}", bit_stream);
    }
}
