#[path = "../constants.rs"]
mod constants;

use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error(transparent)]
#[diagnostic(transparent)]
pub enum LexingError {
    Utoken(#[from] UnrecognizedToken),

    LabelError(#[from] ParseLabelError),

    ValueError(#[from] ParseValueError),

    DefineError(#[from] ParseDefineError),
}

impl Default for LexingError {
    fn default() -> Self {
        LexingError::Utoken(UnrecognizedToken {
            src: NamedSource::new("", String::new()),
            src_span: (0, 1).into(),
        })
    }
}

#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error(transparent)]
#[diagnostic(transparent)]
pub enum ParseDefineError {
    LabelError(#[from] ParseLabelError),

    ValueError(#[from] ParseValueError),

    TooFewOperandError(#[from] DefineFewOperandError),

    TooManyOperandError(#[from] DefineManyOperandError),

    InvalidName(#[from] NameError),
}

#[derive(Error, Debug, Diagnostic, Clone, PartialEq)]
#[error("Name should be of the form [a-zA-Z_]+")]
pub struct NameError {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("here")]
    pub src_span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic, Clone, PartialEq)]
#[error("Too few operands after DEFINE")]
pub struct DefineFewOperandError {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("missing operand for define")]
    pub src_span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic, Clone, PartialEq)]
#[error("Too many operands after DEFINE")]
pub struct DefineManyOperandError {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("additional operand here")]
    pub src_span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic, Clone, PartialEq)]
#[error("Unrecognized Token")]
#[diagnostic(
    code(token_definition::Token),
    help("See the list of tokens in src/lexer/token_definition.rs (todo: give the closest token to the slice given)")
)]
pub struct UnrecognizedToken {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("This doesn't match any token")]
    pub src_span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic, Clone, PartialEq)]
#[error("Multiple Definitions/Labels with the same name")]
#[diagnostic(code(lexer::parse_text_raw))]
pub struct ParseLabelError {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("Can't use this name")]
    pub src_span: SourceSpan,

    #[label("the name is already declared here")]
    pub previous_label_span: SourceSpan,
}

#[derive(Error, Diagnostic, Debug, PartialEq, Clone)]
pub enum ParseValueError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    WrongDigitError(#[from] InvalidDigitError),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    OverflowError(#[from] LoadValueOverflowError),
}

#[derive(Error, Debug, Diagnostic, Clone, PartialEq)]
#[error("Invalid digit found in string")]
#[diagnostic(
    code(lexer::parse_value),
    help("Verify the base prefix and the digits")
)]
pub struct InvalidDigitError {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("Invalid digit here")]
    pub src_span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic, Clone, PartialEq)]
#[error("Value Load Overflow")]
#[diagnostic(
    code(lexer::parse_value),
    help(
        "- The value should be under 0x8000 in hexadecimal\n- The value should be under 32768 in decimal\n- The value should fit in 15 bits\n\nnote: future note on how to quickfix this problem"
    )
)]
pub struct LoadValueOverflowError {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("This value should be under {}", constants::MAX_LOAD_VALUE)]
    pub src_span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
pub enum AppError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    A(LexingError), // todo: change this name
    #[error("Io error")]
    IoError,
}
