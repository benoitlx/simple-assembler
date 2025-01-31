#[path = "../constants.rs"]
mod constants;
mod lexer_error;

use lexer_error::{AppError, LexingError, UnrecognizedToken};
use logos::{Lexer, Logos};
use miette::NamedSource;
use std::io::Read;

pub fn lex_from_file(filename: &str) -> miette::Result<(), AppError> {
    if let Ok(mut file) = std::fs::File::open(filename) {
        let mut content = String::new();

        let _ = file.read_to_string(&mut content);

        let mut lex = Token::lexer_with_extras(content.as_str(), filename.to_owned());

        while let Some(result) = lex.next() {
            match result {
                Ok(token) => println!("{:#?}", token),
                Err(e) => match e {
                    LexingError::Utoken(_) => {
                        Err(AppError::A(LexingError::Utoken(UnrecognizedToken {
                            src: NamedSource::new(filename, content.clone()),
                            src_span: lex.span().into(),
                        })))?
                    }
                    any_error => Err(AppError::A(any_error))?,
                },
            }
        }

        return Ok(());
    }
    Err(AppError::IoError)
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(error = LexingError)]
#[logos(extras = String)]
enum Token {
    // Operations
    #[token("+")]
    #[token("ADD")]
    Add,

    #[token("-")]
    #[token("SUB")]
    Sub,

    #[token("&")]
    #[token("AND")]
    And,

    #[token("|")]
    #[token("OR")]
    Or,

    #[token("^")]
    #[token("XOR")]
    Xor,

    #[token("~")]
    #[token("NOT")]
    Not,

    #[token("<-")]
    Assignment,

    // Branch
    #[token("JMP")]
    Jmp,

    #[token(">")]
    Gt,

    #[token("<")]
    Lt,

    #[token("==")]
    Eq,

    #[token("!=")]
    Neq,

    #[token(">=")]
    Gtoeq,

    #[token("<=")]
    Ltoeq,

    // Registers
    #[token("A")]
    A,

    #[token("*A")]
    StarA,

    #[token("V")]
    V,

    #[token("*V")]
    StarV,

    #[token("C")]
    C,

    // Values
    #[regex("[0-9]+", |lex| parse_value("", 10, lex))]
    #[regex("(0x|0X){1}[a-fA-F0-9]+", |lex| parse_value("0x", 16, lex))]
    #[regex("(0b|0B){1}(0|1)+", |lex| parse_value("0b", 2, lex))]
    Number(u16),

    // Labels
    #[regex("[a-zA-Z_]+:", parse_label)]
    Label(String),
}

fn parse_label(lex: &mut Lexer<Token>) -> Result<String, lexer_error::ParseLabelError> {
    let slice = lex.slice().replace(":", "");

    for maybe_token in lex.clone().spanned() {
        match maybe_token.0 {
            Ok(Token::Label(s)) if s == slice.clone() => Err(lexer_error::ParseLabelError {
                src: NamedSource::new(lex.extras.clone(), lex.source().to_owned()),
                src_span: lex.span().into(),
                previous_label_span: maybe_token.1.into(),
            })?,
            Ok(_) | Err(_) => (),
        }
    }

    Ok(slice)
}

fn parse_value(
    prefix: &str,
    base: u32,
    lex: &mut Lexer<Token>,
) -> Result<u16, lexer_error::ParseValueError> {
    use constants::MAX_LOAD_VALUE;
    use lexer_error::LoadValueOverflowError;
    use lexer_error::ParseValueError;
    use std::num::IntErrorKind::PosOverflow;

    let slice = lex.slice();
    let raw_bits = slice.trim_start_matches(prefix);

    return match u16::from_str_radix(raw_bits, base) {
        Ok(n) if n > MAX_LOAD_VALUE => {
            Err(ParseValueError::OverflowError(LoadValueOverflowError {
                src: NamedSource::new(lex.extras.clone(), lex.source().to_owned()),
                src_span: lex.span().into(),
            }))
        }
        Err(e) if *e.kind() == PosOverflow => {
            println!("value should fir in 16 bits");
            Err(ParseValueError::OverflowError(LoadValueOverflowError {
                src: NamedSource::new(lex.extras.clone(), lex.source().to_owned()),
                src_span: lex.span().into(),
            }))
        }
        Ok(n) => Ok(n),
        Err(e) => Err(ParseValueError::ParseIntError(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_token() {
        let mut lex = Token::lexer("+ ADD");
        assert_eq!(lex.next(), Some(Ok(Token::Add)));
        assert_eq!(lex.next(), Some(Ok(Token::Add)));
    }

    #[test]
    fn test_labels_token() {
        let mut lex = Token::lexer("some_label:");
        assert_eq!(
            lex.next(),
            Some(Ok(Token::Label(String::from("some_label"))))
        );

        let mut lex = Token::lexer("SOME_LABEL:");
        assert_eq!(
            lex.next(),
            Some(Ok(Token::Label(String::from("SOME_LABEL"))))
        );
    }

    #[test]
    fn test_values() {
        let inputs = ["554", "0x5fa4", "0b1000110"];
        let expected_numbers = [554, 0x5fa4, 0b1000110];

        for (l, r) in std::iter::zip(inputs, expected_numbers) {
            let mut lex = Token::lexer(l);

            assert_eq!(lex.next(), Some(Ok(Token::Number(r))))
        }
    }
}
