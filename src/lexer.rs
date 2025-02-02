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

    // Condition has a higher priority than Operation
    // watchout you need to escape the good char
    // tested
    #[regex(r"[\+&=\-|^~]", Op::new, priority = 1)]
    Operation(Op),
    // WIP
    #[regex(r"(==)|(!=)|(<=)|(>=)|<|>|(JMP)", Cond::new, priority = 2)]
    Condition(Cond),

    // WIP
    #[regex(r"[0-9]+", |_| 3)]
    #[regex("(0x|0X){1}[a-fA-F0-9]+", |_| 3)]
    #[regex("(0b|0B){1}(0|1)+", |_| 3)]
    Value(u16),

    // No test
    #[token(":", |_| Dir::Label)]
    #[token("DEFINE", |_| Dir::Define)]
    Directive(Dir),

    // Register has a higher priority than Identifier
    // tested
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

    #[test]
    fn test_register() {
        let mut lex = Token::lexer("A V D *A *V");
        assert_eq!(lex.next(), Some(Ok(Token::Register(Reg::A))));
        assert_eq!(lex.next(), Some(Ok(Token::Register(Reg::V))));
        assert_eq!(lex.next(), Some(Ok(Token::Register(Reg::D))));
        assert_eq!(lex.next(), Some(Ok(Token::Register(Reg::AStar))));
        assert_eq!(lex.next(), Some(Ok(Token::Register(Reg::VStar))));

        let mut lex = Token::lexer("A=");
        assert_eq!(lex.next(), Some(Ok(Token::Register(Reg::A))));

        let mut lex = Token::lexer("A\n");
        assert_eq!(lex.next(), Some(Ok(Token::Register(Reg::A))));

        // the wrong syntax below will be catch by the parser
        let mut lex = Token::lexer("A:");
        assert_eq!(lex.next(), Some(Ok(Token::Register(Reg::A))));

        // the wrong syntax below will be catch by the parser
        let mut lex = Token::lexer("DEFINE A");
        lex.next();
        assert_eq!(lex.next(), Some(Ok(Token::Register(Reg::A))));

        // the wrong syntax below will be catch by the parser
        let mut lex = Token::lexer("DEFINE *A");
        lex.next();
        assert_eq!(lex.next(), Some(Ok(Token::Register(Reg::AStar))));

        // Register B doesn't exist 
        // TODO: return a specific error with helper including the list of valid registers
        let mut lex = Token::lexer("B");
        assert_eq!(lex.next(), Some(Err(())));
    }

    #[test]
    fn test_condition() {
        let mut lex = Token::lexer("== >= <= > < != JMP");
        assert_eq!(lex.next(), Some(Ok(Token::Condition(Cond::Eq))));
        assert_eq!(lex.next(), Some(Ok(Token::Condition(Cond::GtEq))));
        assert_eq!(lex.next(), Some(Ok(Token::Condition(Cond::LtEq))));
        assert_eq!(lex.next(), Some(Ok(Token::Condition(Cond::Gt))));
        assert_eq!(lex.next(), Some(Ok(Token::Condition(Cond::Lt))));
        assert_eq!(lex.next(), Some(Ok(Token::Condition(Cond::Neq))));
        assert_eq!(lex.next(), Some(Ok(Token::Condition(Cond::Jump))));

        let mut lex = Token::lexer("A==A\n");
        lex.next();
        assert_eq!(lex.next(), Some(Ok(Token::Condition(Cond::Eq))));

        let mut lex = Token::lexer("=A==A\n");
        lex.next();
        lex.next();
        assert_eq!(lex.next(), Some(Ok(Token::Condition(Cond::Eq))));
    }

}
