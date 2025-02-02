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

    // Define
    // #[regex("DEFINE [a-zA-Z_]+ [0-9]+", |lex| parse_define("", 10, lex))]
    // #[regex("DEFINE [a-zA-Z_]+ (0x|0X){1}[a-fA-F0-9]+", |lex| parse_define("0x", 16, lex))]
    // #[regex("DEFINE [a-zA-Z_]+ (0b|0B){1}(0|1)+", |lex| parse_define("0b", 2, lex))]
    // #[regex(r"DEFINE\s*[^\s]*", define_too_few_arguments)]
    #[regex(r"DEFINE [^[\n(//)]]*", parse_define)] // tofix: wrong error with "DEFINE t/est 0x0"
    Define((String, u16)),

    // Comments
    // #[regex(r"\s*/*.**/")] // multiline comments
    #[regex(r"\s*//.*")]
    Comments,
}

fn parse_label(lex: &mut Lexer<Token>) -> Result<String, lexer_error::ParseLabelError> {
    // check for regex [a-zA-Z_]+
    // if it fail => NameError

    let slice = lex.slice().replace(":", "");

    parse_text_raw(
        slice,
        lex.span(),
        lex.extras.clone(),
        lex.source(),
        lex.clone().spanned(),
    )
}

fn parse_text_raw(
    slice: String,
    span: logos::Span,
    f: String,
    source: &str,
    spanned: logos::SpannedIter<Token>,
) -> Result<String, lexer_error::ParseLabelError> {
    for maybe_token in spanned {
        match maybe_token.0 {
            Ok(Token::Label(s)) if s == slice.clone() => Err(lexer_error::ParseLabelError {
                src: NamedSource::new(&f, source.to_owned()),
                src_span: span.clone().into(),
                previous_label_span: maybe_token.1.into(),
            })?,
            Ok(Token::Define((s, _))) if s == slice.clone() => Err(lexer_error::ParseLabelError {
                src: NamedSource::new(&f, source.to_owned()),
                src_span: span.clone().into(),
                previous_label_span: (maybe_token.1.start + 7, s.len()).into(),
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
    let slice = lex.slice();

    parse_value_raw(
        prefix,
        base,
        slice,
        lex.span(),
        lex.extras.clone(),
        lex.source(),
    )
}

fn parse_value_raw(
    prefix: &str,
    base: u32,
    slice: &str,
    span: logos::Span,
    f: String,
    s: &str,
) -> Result<u16, lexer_error::ParseValueError> {
    use constants::MAX_LOAD_VALUE;
    use lexer_error::InvalidDigitError;
    use lexer_error::LoadValueOverflowError;
    use lexer_error::ParseValueError;
    use std::num::IntErrorKind::InvalidDigit;
    use std::num::IntErrorKind::PosOverflow;

    let raw_bits = slice.trim_start_matches(prefix);

    return match u16::from_str_radix(raw_bits, base) {
        Ok(n) if n > MAX_LOAD_VALUE => {
            Err(ParseValueError::OverflowError(LoadValueOverflowError {
                src: NamedSource::new(f, s.to_owned()),
                src_span: span.into(),
            }))
        }
        Err(e) if *e.kind() == PosOverflow => {
            Err(ParseValueError::OverflowError(LoadValueOverflowError {
                src: NamedSource::new(f, s.to_owned()),
                src_span: span.into(),
            }))
        }
        Err(e) if *e.kind() == InvalidDigit => {
            Err(ParseValueError::WrongDigitError(InvalidDigitError {
                src: NamedSource::new(f, s.to_owned()),
                src_span: span.into(),
            }))
        }
        Ok(n) => Ok(n),
        Err(e) => Err(ParseValueError::ParseIntError(e)),
    };
}

fn parse_define(lex: &mut Lexer<Token>) -> Result<(String, u16), lexer_error::ParseDefineError> {
    use lexer_error::DefineFewOperandError;
    use lexer_error::DefineManyOperandError;
    use lexer_error::NameError;
    use lexer_error::ParseDefineError;
    use regex::Regex;

    let mut slices = lex.slice().trim_end().split_whitespace();
    let mut result: (String, u16) = (String::from(""), 0);

    let _ = slices.next();

    let mut arg_number = 0;
    if let Some(label) = slices.next() {
        arg_number += 1;

        let label_length = label.len();
        let span_start = lex.span().start + 7;
        let span_range = span_start..(span_start + label_length);

        let re = Regex::new(r"^[a-zA-Z_]+$").unwrap();

        if !re.is_match(label) {
            return Err(ParseDefineError::InvalidName(NameError {
                src: NamedSource::new(lex.extras.clone(), lex.source().to_owned()),
                src_span: span_range.into(),
            }));
        }

        result.0 = parse_text_raw(
            label.to_owned(),
            span_range,
            lex.extras.clone(),
            lex.source(),
            lex.clone().spanned(),
        )?;

        if let Some(value) = slices.next() {
            let mut prefix = "";
            let mut base = 10;

            if value.len() >= 2 && &value[0..2] == "0x" {
                prefix = "0x";
                base = 16;
            }
            if value.len() >= 2 && &value[0..2] == "0b" {
                prefix = "0b";
                base = 2;
            }

            arg_number += 1;

            let value_length = value.len();

            result.1 = parse_value_raw(
                prefix,
                base,
                value,
                (lex.span().start + 8 + label_length)
                    ..(lex.span().start + 8 + label_length + value_length),
                lex.extras.clone(),
                lex.source(),
            )?;
        }
    }

    if slices.next() != None {
        return Err(ParseDefineError::TooManyOperandError(
            DefineManyOperandError {
                src: NamedSource::new(lex.extras.clone(), lex.source().to_owned()),
                src_span: lex.span().into(),
            },
        ));
    }

    if arg_number != 2 {
        return Err(ParseDefineError::TooFewOperandError(
            DefineFewOperandError {
                src: NamedSource::new(lex.extras.clone(), lex.source().to_owned()),
                src_span: lex.span().into(),
            },
        ));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::lexer::lexer_error::ParseDefineError;

    use super::*;

    #[test]
    fn test_add_token() {
        let mut lex = Token::lexer("+ ADD");
        assert_eq!(lex.next(), Some(Ok(Token::Add)));
        assert_eq!(lex.next(), Some(Ok(Token::Add)));
    }

    #[test]
    fn test_labels() {
        use LexingError::LabelError;

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

        // wrong syntax

        // Multiple use of the same name
        let mut lex = Token::lexer("test:\nDEFINE test 0");
        assert!(matches!(lex.next(), Some(Err(LabelError(_)))));

        let mut lex = Token::lexer("test:\ntest:");
        assert!(matches!(lex.next(), Some(Err(LabelError(_)))));

        // wrong label name
        /*         let mut lex = Token::lexer("te/st:");
        assert!(matches!(lex.next(), Some(Err(LabelError(_))))); // to fix: return Err(LexingError::Utoken) */
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

    #[test]
    fn test_defines() {
        use LexingError::DefineError;
        use Token::Define;
        use ParseDefineError::LabelError;
        use ParseDefineError::ValueError;

        // good syntax
        let mut lex = Token::lexer("DEFINE test 0");
        assert_eq!(lex.next(), Some(Ok(Define((String::from("test"), 0)))));

        let mut lex = Token::lexer("DEFINE test 1");
        assert_eq!(lex.next(), Some(Ok(Define((String::from("test"), 1)))));

        let mut lex = Token::lexer("DEFINE test 32767");
        assert_eq!(lex.next(), Some(Ok(Define((String::from("test"), 32767)))));

        let mut lex = Token::lexer("DEFINE test 0x0");
        assert_eq!(lex.next(), Some(Ok(Define((String::from("test"), 0)))));

        let mut lex = Token::lexer("DEFINE test 0xff");
        assert_eq!(lex.next(), Some(Ok(Define((String::from("test"), 255)))));

        let mut lex = Token::lexer("DEFINE test 0x7fff");
        assert_eq!(lex.next(), Some(Ok(Define((String::from("test"), 32767)))));

        let mut lex = Token::lexer("DEFINE test 0b0");
        assert_eq!(lex.next(), Some(Ok(Define((String::from("test"), 0)))));

        let mut lex = Token::lexer("DEFINE TOTO 0b11");
        assert_eq!(lex.next(), Some(Ok(Define((String::from("TOTO"), 3)))));

        let mut lex = Token::lexer("DEFINE titi_test 0b111111111111111");
        assert_eq!(
            lex.next(),
            Some(Ok(Define((String::from("titi_test"), 32767))))
        );

        // wrong syntax

        // Multiple use of the same name
        let mut lex = Token::lexer("DEFINE test 0\ntest:");
        assert!(matches!(lex.next(), Some(Err(DefineError(LabelError(_))))));

        let mut lex = Token::lexer("DEFINE test 0\nDEFINE test 0");
        assert!(matches!(lex.next(), Some(Err(DefineError(LabelError(_))))));

        // Value Error
        let mut lex = Token::lexer("DEFINE test 0feaj138");
        assert!(matches!(lex.next(), Some(Err(DefineError(ValueError(_))))));

        let mut lex = Token::lexer("DEFINE test 0x8000"); // load value overflow
        assert!(matches!(lex.next(), Some(Err(DefineError(ValueError(_))))));
    }
}
