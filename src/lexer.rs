use logos::{Lexer, Logos};

/* >> Architecture being used << */
#[path = "spec.rs"]
mod spec;
use spec::arch_v1::*;

/// This trait is either used by the lexer to produce Token with the new method
/// or by the parser to generate the bit stream from a Token
pub trait HandleToken {
    // get the bit stream from an item (Reg, Op, Inst)
    fn bit_stream(&self) -> String {
        String::new()
    }

    fn new(lex: &mut Lexer<Token>) -> Option<Self>
    where
        Self: Sized; // todo: default implementation qui renvoit une erreur en spécifiant le type Self
}

macro_rules! parse_number {
    ($name:ident, $prefix:expr, $radix:expr) => {
        fn $name(lex: &mut Lexer<Token>) -> Option<u16> {
            let raw_slice = lex.slice().trim_start_matches($prefix);
            match u16::from_str_radix(raw_slice, $radix) {
                Ok(n) if n <= MAX_LOAD_VALUE => Some(n),
                Ok(_) | Err(_) => None,
            }
        }
    };
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token {
    // Condition has a higher priority than Operation
    // watchout you need to escape the good char
    // tested
    #[regex(r"[\+&=\-|^~]", Op::new, priority = 1)]
    Operation(Op),
    // tested
    #[regex(r"(==)|(!=)|(<=)|(>=)|<|>|(JMP)", Cond::new, priority = 2)]
    Condition(Cond),

    // tested 
    #[regex(r"[0-9]+", Token::decimal)]
    #[regex("0x[a-fA-F0-9]+", Token::hexadecimal)]
    #[regex("0b(0|1)+", Token::binary)]
    Value(u16),

    // tested
    #[token(":", Dir::new)]
    #[token("DEFINE", Dir::new)]
    Directive(Dir),

    // Register has a higher priority than Identifier
    // tested
    #[regex(r"\*?[A-Z]", Reg::new, priority = 2)]
    Register(Reg),
    // tested 
    #[regex(r"[a-z_A-Z]+", Token::text, priority = 1)]
    Identifier(String),

    // No test
    #[regex(r";[^\n]*")]
    Comment,
}

impl Token {
    parse_number!(decimal, "", 10);
    parse_number!(hexadecimal, "0x", 16);
    parse_number!(binary, "0b", 2);

    fn text(lex: &mut Lexer<Token>) -> Option<String> {
        Some(lex.slice().to_string())
    }
}

#[derive(Debug, PartialEq)]
pub enum Dir {
    Define,
    Label,
}

impl HandleToken for Dir {
    fn new(lex: &mut Lexer<Token>) -> Option<Self>
    where
        Self: Sized,
    {
        match lex.slice() {
            "DEFINE" => Some(Dir::Define),
            ":" => Some(Dir::Label),
            _ => None,
        }
    }
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

    #[test]
    fn test_directive() {
        let mut lex = Token::lexer("DEFINE label 0x0\ntest:");
        assert_eq!(lex.next(), Some(Ok(Token::Directive(Dir::Define))));
        lex.next();
        lex.next();
        lex.next();
        assert_eq!(lex.next(), Some(Ok(Token::Directive(Dir::Label))));

        let mut lex = Token::lexer("DEFINE:\n");
        assert_eq!(lex.next(), Some(Ok(Token::Directive(Dir::Define))));
        assert_eq!(lex.next(), Some(Ok(Token::Directive(Dir::Label))));
    }

    #[test]
    fn test_values() {
        let mut lex = Token::lexer("0 1 32767 0x0 0x1 0x7fff 0b0 0b1 0b111111111111111\n");
        assert_eq!(lex.next(), Some(Ok(Token::Value(0))));
        assert_eq!(lex.next(), Some(Ok(Token::Value(1))));
        assert_eq!(lex.next(), Some(Ok(Token::Value(32767))));
        assert_eq!(lex.next(), Some(Ok(Token::Value(0))));
        assert_eq!(lex.next(), Some(Ok(Token::Value(1))));
        assert_eq!(lex.next(), Some(Ok(Token::Value(32767))));
        assert_eq!(lex.next(), Some(Ok(Token::Value(0))));
        assert_eq!(lex.next(), Some(Ok(Token::Value(1))));
        assert_eq!(lex.next(), Some(Ok(Token::Value(32767))));

        let mut lex = Token::lexer("32768 0x8000 0b1000000000000000");
        assert_eq!(lex.next(), Some(Err(())));
        assert_eq!(lex.next(), Some(Err(())));
        assert_eq!(lex.next(), Some(Err(())));
    }

    #[test]
    fn test_identifier() {
        let mut lex = Token::lexer("DEFINE id 0x0\nid:");
        lex.next();
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("id".to_string()))));
        lex.next();
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("id".to_string()))));

        let test_string = "a b c foo bar FOO BAR foo_bar FOO_BAR Foo_Bar";
        let string_iter = test_string.split(" ");
        let mut lex = Token::lexer(test_string);

        for word in string_iter {
            assert_eq!(lex.next(), Some(Ok(Token::Identifier(word.to_string()))));
        }
    }
}
