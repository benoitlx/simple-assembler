use crate::constants::MAX_LOAD_VALUE;
use logos::{Lexer, Logos};

fn parse_values(prefix: &str, base: u32, lex: &mut Lexer<Token>) -> Option<u16> {
    let slice = lex.slice();
    let raw_bits = slice.trim_start_matches(prefix);
    let n: u16 = u16::from_str_radix(raw_bits, base).ok()?;
    assert!(
        n <= MAX_LOAD_VALUE,
        "Can't load data exceeding {} from ram",
        MAX_LOAD_VALUE
    );
    Some(n)
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
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
    #[regex("[0-9]+", |lex| parse_values("", 10, lex))]
    #[regex("(0x|0X){1}[a-fA-F0-9]+", |lex| parse_values("0x", 16, lex))]
    #[regex("(0b|0B){1}(0|1)+", |lex| parse_values("0b", 2, lex))]
    Number(u16),

    // Labels
    #[regex("[a-zA-Z_]+:", |lex| lex.slice().replace(":", ""))]
    Label(String),
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
    fn test_not_a_label() {
        let mut lex = Token::lexer("Centrale Lille");

        assert_eq!(lex.next(), Some(Err(())));
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
