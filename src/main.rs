mod lexer;
mod parser;

use std::fs::File;

use clap::Parser;
use colored::Colorize;
use lexer::Token;
use logos::Logos;
use miette::Severity;

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

    /// whether to turn off warnings
    #[arg(long = "w-off")]
    warning_off: bool,

    /// whether to output the bit stream if warnings are encountered
    #[arg(short = 'W', long = "Warn")]
    warning: bool,

    /// save output in designated file
    #[arg(short = 'o', long = "output")]
    output_path: Option<String>,
}

fn main() {
    use std::io::Read;
    use std::io::Write;

    let args = Cli::parse();

    if let Ok(mut file) = std::fs::File::open(args.file_path.clone()) {
        let mut content = String::new();
        let _ = file.read_to_string(&mut content);

        let lex = Token::lexer(content.as_str());

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        let parser_report = parser::parse(&mut tokens, args.color, args.debug, &args.sep);

        let (errors, warnings): (Vec<_>, Vec<_>) = parser_report
            .report
            .into_iter()
            .partition(|r| r.severity() != Some(Severity::Warning));

        let error_number = errors.len();
        let warning_number = warnings.len();

        if warning_number > 0 && !args.warning_off {
            for w in warnings {
                println!("{:?}", w.with_source_code(content.clone()));
            }
        }

        if error_number > 0 {
            for e in errors {
                println!("{:?}", e.with_source_code(content.clone()));
            }

            println!(
                "{} errors and {} warnings found in {}, exiting !",
                error_number, warning_number, args.file_path
            );
            return;
        }

        if warning_number > 0 && !args.warning {
            println!("{} warnings encountered, exiting !", warning_number);

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

        if let Some(path) = args.output_path {
            let mut output = File::create(path).unwrap();
            let _ = write!(output, "{}", parser_report.bit_stream);
        } else {
            println!("{}", parser_report.bit_stream);
        }
    }
}
