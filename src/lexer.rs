use logos::{Lexer, Logos};

/* >> Architecture being used << */
#[path ="spec.rs"]
mod spec;
use spec::arch_v1::*;

/// This trait is either used by the lexer to produce Token with the new method
/// or by the parser to generate the bit stream from a Token
pub trait HandleToken {
    fn bit_stream(&self) -> String; // get the bit stream from an item (Reg, Op, Inst)

    fn new(lex: &mut Lexer<Token>) -> Option<Self>
    where Self: Sized; // todo: default implementation qui renvoit une erreur en spécifiant le type Self
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token {

    // watchout you need to escape the good char
    #[regex(r"[\+&=\-|^~]", Op::new)]
    Operation(Op),

    // WIP
    #[regex(r"==", |_| Cond::Eq)]
    Condition(Cond),

    // WIP
    #[regex(r"[0-9]+", |_| 3)]
    #[regex("(0x|0X){1}[a-fA-F0-9]+", |_| 3)]
    #[regex("(0b|0B){1}(0|1)+", |_| 3)]
    Value(u16),

    // WIP
    #[token("JMP", |_| Inst::Jump)]
    Instruction(Inst),

    // No test
    #[token(":", |_| Dir::Label)]
    #[token("DEFINE", |_| Dir::Define)]
    Directive(Dir),

    // Register has a higher priority than Identifier
    #[regex(r"\*?[A-Z]", Reg::new, priority = 2)]
    Register(Reg),
    // No test
    #[regex(r"[a-z_A-Z]+", |lex| lex.slice().to_string(), priority = 1)]
    Identifier(String),

    // No test
    #[regex(r";[^\n]*")]
    Comment,
}

#[derive(Debug, PartialEq)]
pub enum Dir {
    Define,
    Label,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation() {
        let mut lex = Token::lexer("+~-&|^=");
        
        assert_eq!(lex.next(), Some(Ok(Token::Operation(Op::Add))));
        assert_eq!(lex.next(), Some(Ok(Token::Operation(Op::Not))));
        assert_eq!(lex.next(), Some(Ok(Token::Operation(Op::Sub))));
        assert_eq!(lex.next(), Some(Ok(Token::Operation(Op::And))));
        assert_eq!(lex.next(), Some(Ok(Token::Operation(Op::Or))));
        assert_eq!(lex.next(), Some(Ok(Token::Operation(Op::Xor))));
        assert_eq!(lex.next(), Some(Ok(Token::Operation(Op::Assignement))));

        let mut lex = Token::lexer(" +");
        assert_eq!(lex.next(), Some(Ok(Token::Operation(Op::Add))));

        let mut lex = Token::lexer("+ \n");
        assert_eq!(lex.next(), Some(Ok(Token::Operation(Op::Add))));

        let mut lex = Token::lexer("A+A\n");
        lex.next();
        assert_eq!(lex.next(), Some(Ok(Token::Operation(Op::Add))));
        
        let mut lex = Token::lexer("A +A\n");
        lex.next();
        assert_eq!(lex.next(), Some(Ok(Token::Operation(Op::Add))));
    }

}
