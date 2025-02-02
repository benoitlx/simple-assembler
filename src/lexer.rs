use logos::{Lexer, Logos};

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
pub enum Op {
    Add,
    Sub,
    And,
    Assignement,
}

impl Op {
    fn new(lex: &mut Lexer<Token>) -> Option<Op> {
        match lex.slice().trim() {
            "+" => Some(Op::Add),
            "-" => Some(Op::Sub),
            "&" => Some(Op::And),
            "=" => Some(Op::Assignement),
            _ => None, // todo: return a beautiful error
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Reg {
    A,
    V,
    AStar,
    VStar,
    D,
}

impl Reg {
    fn new(lex: &mut Lexer<Token>) -> Option<Reg> {
        match lex.slice() {
            "A" => Some(Reg::A),
            "V" => Some(Reg::V),
            "*A" => Some(Reg::AStar),
            "*V" => Some(Reg::VStar),
            "D" => Some(Reg::D),
            _ => None, // todo: return a beautiful error
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Cond {
    Eq,
    Neq,
}

#[derive(Debug, PartialEq)]
pub enum Inst {
    Jump,
}

#[derive(Debug, PartialEq)]
pub enum Dir {
    Define,
    Label,
}
