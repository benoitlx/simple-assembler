pub mod arch_v1 {
    use crate::lexer::HandleToken;
    use logos::Lexer;

    #[allow(dead_code)]

    pub const MAX_LOAD_VALUE: u16 = 2_u16.pow(15) - 1;

    #[derive(Debug, PartialEq)]
    pub enum Op {
        Add,
        Sub,
        And,
        Not,
        Or,
        Xor,
        Assignement,
    }

    impl HandleToken for Op {
        fn new(lex: &mut Lexer<crate::lexer::Token>) -> Option<Op> {
            match lex.slice().trim() {
                "+" => Some(Op::Add),
                "-" => Some(Op::Sub),
                "&" => Some(Op::And),
                "~" => Some(Op::Not),
                "|" => Some(Op::Or),
                "^" => Some(Op::Xor),
                "=" => Some(Op::Assignement),
                _ => None, // todo: return a beautiful error
            }
        }

        fn bit_stream(&self) -> String {
            todo!();
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

    impl HandleToken for Reg {
        fn new(lex: &mut Lexer<crate::lexer::Token>) -> Option<Reg> {
            match lex.slice() {
                "A" => Some(Reg::A),
                "V" => Some(Reg::V),
                "*A" => Some(Reg::AStar),
                "*V" => Some(Reg::VStar),
                "D" => Some(Reg::D),
                _ => None, // todo: return a beautiful error
            }
        }

        fn bit_stream(&self) -> String {
            match self {
                Reg::A => "000",
                Reg::AStar => "001",
                Reg::V => "010",
                Reg::VStar => "011",
                Reg::D => "100",
            }
            .to_string()
        } // todo: colorize the string depending on the register
    }

    #[derive(Debug, PartialEq)]
    pub enum Cond {
        Eq,
        Neq,
        Gt,
        Lt,
        GtEq,
        LtEq,
        Jump,
    }

    impl HandleToken for Cond {
        fn new(lex: &mut Lexer<crate::lexer::Token>) -> Option<Self>
        where
            Self: Sized,
        {
            match lex.slice() {
                "==" => Some(Cond::Eq),
                ">" => Some(Cond::Gt),
                "<" => Some(Cond::Lt),
                ">=" => Some(Cond::GtEq),
                "<=" => Some(Cond::LtEq),
                "!=" => Some(Cond::Neq),
                "JMP" => Some(Cond::Jump),
                _ => None, // todo error
            }
        }

        fn bit_stream(&self) -> String {
            todo!();
        }
    }
}
