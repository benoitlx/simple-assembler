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
#[logos(skip r"\s+")]
pub enum Token {
    #[regex(r" [+-~&|^=] ", Op::new, priority = 8)]
    Operation(Op),

    #[regex(r" [(==)<>(>=)(<=)(!=)] ", |_| Cond::Eq, priority = 7)]
    Condition(Cond),

    #[regex(r"[0-9]+", |_| 3, priority = 6)]
    #[regex("(0x|0X){1}[a-fA-F0-9]+", |_| 3, priority = 6)]
    #[regex("(0b|0B){1}(0|1)+", |_| 3, priority = 6)]
    Value(u16),

    #[token("JMP", |_| Inst::Jump, priority = 5)]
    Instruction(Inst),

    #[token(":", |_| Dir::Label, priority = 4)]
    #[token("DEFINE", |_| Dir::Define, priority = 4)]
    Directive(Dir),

    #[regex(r"\*?[A-Z]{1}", Reg::new, priority = 3)]
    Register(Reg),

    #[regex(r"[a-zA-Z_]+", |lex| lex.slice().to_string(), priority = 1)]
    Identifier(String),

    #[regex(r"\s?;.*")]
    Comment,
}

#[derive(Debug, PartialEq)]
pub enum Dir {
    Define,
    Label,
}
