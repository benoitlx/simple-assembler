pub mod arch_v1 {
    use crate::lexer::HandleToken;
    use logos::Lexer;
    use colored::Colorize;

    #[allow(dead_code)]

    pub const MAX_LOAD_VALUE: u16 = 2_u16.pow(15) - 1;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Op {
        Add,
        Sub,
        And,
        Not,
        Or,
        Xor,
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
                _ => None, // todo: return a beautiful error
            }
        }

        fn bit_stream(&self) -> String {
            match self {
                Op::Add => "000",
                Op::Sub => "001",
                Op::And => "010",
                Op::Or => "011",
                Op::Xor => "100",
                Op::Not => "101"
            }.to_string()
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Reg {
        A,
        V,
        AStar,
        VStar,
        D,
        Zero,
        One
    }

    impl HandleToken for Reg {
        fn new(lex: &mut Lexer<crate::lexer::Token>) -> Option<Reg> {
            match lex.slice() {
                "A" => Some(Reg::A),
                "V" => Some(Reg::V),
                "*A" => Some(Reg::AStar),
                "*V" => Some(Reg::VStar),
                "D" => Some(Reg::D),
                "Z" => Some(Reg::Zero),
                "O" => Some(Reg::One),
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
                Reg::Zero => "110",
                Reg::One => "111",
            }
            .to_string()
        } // todo: colorize the string depending on the register
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
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
            match self {
                Cond::Eq => "010",
                Cond::Neq => "101",
                Cond::Gt => "001",
                Cond::Lt => "100",
                Cond::GtEq => "011",
                Cond::LtEq => "110",
                Cond::Jump => "111",
            }
            .to_string()
        }
    }


    // wrapper around Op and Cond
    #[allow(dead_code)]
    pub enum OpOrCond {
        Operation(Op),
        Condition(Cond),
    }

    impl HandleToken for OpOrCond {
        fn new(_lex: &mut Lexer<crate::lexer::Token>) -> Option<Self>
            where
                Self: Sized {
            None
        } 

        fn bit_stream(&self) -> String {
            match self {
                OpOrCond::Operation(op) => op.bit_stream(),
                OpOrCond::Condition(cond) => cond.bit_stream(),
            }
        }
    }

    /// takes a 15 bits value and format it in a recognizable word for the cpu
    #[allow(dead_code)]
    pub fn data_mode_format(val: u16) -> String {
        format!("{}{}", "1".green(), format!("{:015b}", val).red())
    }

    /// takes operands operation and destination register and format it in a recognizable word for the cpu
    #[allow(dead_code)]
    pub fn inst_mode_format(op_or_cond: OpOrCond, rega: Reg, regb: Reg, regc: Reg) -> String {
        format!(
            "{}{}000{}{}{}",
            "0".green().bold(),
            op_or_cond.bit_stream().blue(),
            rega.bit_stream().yellow(),
            regb.bit_stream().purple(),
            regc.bit_stream().cyan()
        )
    }
}
