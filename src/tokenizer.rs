use logos::{Lexer, Logos};
use crate::constants::MAX_LOAD_VALUE;

pub fn parse_int(lex: &mut Lexer<Token>) -> Option<u16> {
    let slice = lex.slice();
    let n: u16 = slice.parse().ok()?;
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

    #[token("<-")]
    Assignment,

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
    #[regex("[0-9]+", parse_int)]
    Number(u16),
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
}