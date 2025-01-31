mod lexer_error;

use logos::Logos;
use lexer_error::LexingError;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(error = LexingError)]
#[logos(extras = String)]
// #[logos(extras = (filename, contents))]
// see: https://docs.rs/logos/latest/logos/trait.Logos.html
pub enum Token {
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
    // #[regex("[a-zA-Z_]+:", |lex| lex.slice().replace(":", ""))]
    #[regex("[a-zA-Z_]+:", parse_label)]
    Label(String),
}