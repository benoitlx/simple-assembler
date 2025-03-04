use crate::lexer::spec::arch_v1::*;
use crate::lexer::{Token::*, *};
use colored::Colorize;
use miette::{miette, Error, LabeledSpan, Severity};
use std::collections::HashMap;
use std::ops::Range;

/* This should be placed in spec ======================================== */
trait BitStream {
    fn bit_stream(&self) -> String;
}

enum OpOrCond {
    Operation(Op),
    Condition(Cond),
}

impl BitStream for OpOrCond {
    fn bit_stream(&self) -> String {
        match self {
            OpOrCond::Operation(op) => op.bit_stream(),
            OpOrCond::Condition(cond) => cond.bit_stream(),
        }
    }
}

/// takes a 15 bits value and format it in a recognizable word for the cpu
fn data_mode_format(val: u16) -> String {
    format!("{}{}", "1".green(), format!("{:015b}", val).red())
}

/// takes operands operation and destination register and format it in a recognizable word for the cpu
fn inst_mode_format(op_or_cond: OpOrCond, rega: Reg, regb: Reg, regc: Reg) -> String {
    format!(
        "{}{}000{}{}{}",
        "0".green().bold(),
        op_or_cond.bit_stream().blue(),
        rega.bit_stream().yellow(),
        regb.bit_stream().purple(),
        regc.bit_stream().cyan()
    )
}
/* ============================================================= */

#[derive(PartialEq, Debug, Clone)]
pub struct ColType {
    val: u16,
    span: Range<usize>,
    visited: bool,
}

pub struct ParserReport {
    pub bit_stream: String,
    pub report: Vec<Error>,

    // this field is only used for test purpose
    #[allow(dead_code)]
    id_collect: HashMap<String, ColType>,
}

/// generate a bit stream from a Vec of Spanned Token
pub fn parse(
    tokens: &mut Vec<(Result<Token, ()>, Range<usize>)>,
    colorize: bool,
    debug: bool,
    sep: &str,
) -> ParserReport {
    colored::control::set_override(colorize);
    let mut bit_stream_with_id: Vec<String> = vec![];

    let mut errors: Vec<Error> = vec![];

    // Hashmap for the identifiers
    let mut id_collect: HashMap<String, ColType> = HashMap::new();
    let mut adr = 0;

    let mut i = 0;
    let n = tokens.len();

    // fill the token vec with 5 comments
    for _ in 0..5 {
        tokens.push((Ok(Token::Comment), 0..0));
    }

    while i < n {
        let tokens_window = &tokens[i..(i + 5)];

        if debug {
            println!("{i}, {:?}", tokens_window);
        }

        let inst_word = match tokens_window {
            // A <- D & *A
            [(Ok(Register(regc)), spanc), (Ok(Assignement), _), (Ok(Register(rega)), spana), (Ok(Operation(op)), spanop), (Ok(Register(regb)), spanb)] =>
            {
                if *regc == Reg::A && (*rega == Reg::AStar || *regb == Reg::AStar) {
                    let report = miette!(
                        labels = vec![
                            LabeledSpan::at(spanc.clone(), "This"),
                            LabeledSpan::at(
                                (spana.clone()).start..(spanb.clone().end),
                                "and this are incompatible"
                            ),
                        ],
                        "Error Can't change A value when reading *A"
                    );
                    errors.push(report);
                }
                if *regc == Reg::V && (*rega == Reg::VStar || *regb == Reg::VStar) {
                    let report = miette!(
                        labels = vec![
                            LabeledSpan::at(spanc.clone(), "This"),
                            LabeledSpan::at(
                                (spana.clone()).start..(spanb.clone().end),
                                "and this are incompatible"
                            ),
                        ],
                        "Error Can't change V value when reading *V"
                    );
                    errors.push(report);
                }
                if *op == Op::Not {
                    let report = miette!(
                        labels = vec![
                            LabeledSpan::at(spana.clone(), "Excessive operand"),
                            LabeledSpan::at(spanop.clone(), "with operation ~"),
                        ],
                        help = format!("Try removing {:?}", *rega),
                        "Error Too many operand for ~"
                    );
                    errors.push(report);
                }
                i += 5;
                adr += 16;
                inst_mode_format(OpOrCond::Operation(*op), *rega, *regb, *regc)
            }
            // A <- mask, tested
            [(Ok(Register(regc)), span), (Ok(Assignement), _), (Ok(Identifier(id)), spanid), _, _] =>
            {
                if *regc != Reg::A {
                    let report = miette!(
                        labels = vec![LabeledSpan::at(span.clone(), "This should be A")],
                        help = format!("Consider using this: \nA = {id}\n{:?} = A", *regc),
                        "Error Can't push direct value into other register than A"
                    );
                    errors.push(report);
                }

                i += 3;
                adr += 16;
                format!("{}#{}#{}#", id, spanid.start, spanid.end)
            }
            // A <- 0x7fff, tested
            [(Ok(Register(regc)), span), (Ok(Assignement), _), (Ok(Value(val)), _), _, _] => {
                if *regc != Reg::A {
                    let report = miette!(
                        labels = vec![LabeledSpan::at(span.clone(), "This should be A")],
                        help = format!("Consider using this: \nA = {val}\n{:?} = A", *regc),
                        "Error Can't push direct value into other register than A"
                    );
                    errors.push(report);
                }

                i += 3;
                adr += 16;
                data_mode_format(*val)
            }
            // A <- D, tested
            [(Ok(Register(regc)), spanc), (Ok(Assignement), _), (Ok(Register(rega)), spana), _, _] =>
            {
                if *regc == Reg::A && *rega == Reg::AStar {
                    let report = miette!(
                        labels = vec![
                            LabeledSpan::at(spanc.clone(), "This"),
                            LabeledSpan::at(spana.clone(), "and this are incompatible"),
                        ],
                        "Error Can't change A value when reading *A"
                    );
                    errors.push(report);
                }
                if *regc == Reg::V && *rega == Reg::VStar {
                    let report = miette!(
                        labels = vec![
                            LabeledSpan::at(spanc.clone(), "This"),
                            LabeledSpan::at(spana.clone(), "and this are incompatible"),
                        ],
                        "Error Can't change V value when reading *V"
                    );
                    errors.push(report);
                }
                i += 3;
                adr += 16;
                inst_mode_format(OpOrCond::Operation(Op::Or), *rega, Reg::Zero, *regc)
            }
            // A <- ~D, tested
            [(Ok(Register(regc)), _), (Ok(Assignement), _), (Ok(Operation(op)), _), (Ok(Register(rega)), _), _] =>
            {
                if *op != Op::Not {
                    panic!("Expected a not operation");
                }
                if *regc == Reg::A && *rega == Reg::AStar {
                    panic!("Cannot change A value when reading *A");
                }
                if *regc == Reg::V && *rega == Reg::VStar {
                    panic!("Cannot change V value when reading *V");
                }
                i += 4;
                adr += 16;
                inst_mode_format(OpOrCond::Operation(*op), *rega, Reg::A, *regc)
            }
            // D>=, tested
            /*
            A <- main
            D>= (<=> D >= 0 ?)
            JMP
             */
            [(Ok(Register(rega)), _), (Ok(Condition(cond)), _), _, _, _] => {
                i += 2;
                adr += 16;
                inst_mode_format(OpOrCond::Condition(*cond), *rega, Reg::Zero, Reg::Zero)
            }
            // JMP, tested
            [(Ok(Condition(Cond::Jump)), _), _, _, _, _] => {
                i += 1;
                adr += 16;
                format!("0111000000000000")
            }
            // label:, tested
            [(Ok(Identifier(id)), span), (Ok(Directive(Dir::Label)), _), _, _, _] => {
                i += 2;

                let id_ref = id_collect.get(id);

                if id_ref == None {
                    id_collect.insert(
                        id.clone(),
                        ColType {
                            val: adr,
                            span: span.clone(),
                            visited: false,
                        },
                    );
                } else {
                    let other_span = (*id_ref.unwrap()).span.clone();
                    let report = miette!(
                        labels = vec![
                            LabeledSpan::at(other_span, "previously declared here"),
                            LabeledSpan::at(span.clone(), "declared there"),
                        ],
                        "Error: Identifier already declared"
                    );
                    errors.push(report);
                }

                String::new()
            }
            // DEFINE mask 0x1, tested
            [(Ok(Directive(Dir::Define)), _), (Ok(Identifier(id)), span), (Ok(Value(val)), _), _, _] =>
            {
                i += 3;

                let id_ref = id_collect.get(id);

                if id_ref == None {
                    id_collect.insert(
                        id.clone(),
                        ColType {
                            val: *val,
                            span: span.clone(),
                            visited: false,
                        },
                    );
                } else {
                    let other_span = (*id_ref.unwrap()).span.clone();
                    let report = miette!(
                        labels = vec![
                            LabeledSpan::at(other_span, "previously declared here"),
                            LabeledSpan::at(span.clone(), "declared there"),
                        ],
                        "Error: Identifier already declared"
                    );
                    errors.push(report);
                }

                String::new()
            }
            [(Ok(Comment), _), _, _, _, _] => {
                i += 1;
                String::new()
            }
            _ => panic!("Unexpected Error"),
        };

        if inst_word != "" {
            bit_stream_with_id.push(inst_word);
        }
    }

    if debug {
        println!("{:.?}", id_collect);
    }

    fn handle_id(
        word: String,
        col: &mut HashMap<String, ColType>,
        errs: &mut Vec<Error>,
    ) -> String {
        if word.chars().last().unwrap() == '#' {
            let mut splited_word = word.split("#");
            let id = splited_word.next().unwrap();
            let start: usize = splited_word
                .next()
                .expect("unable to parse span")
                .parse()
                .unwrap();
            let end: usize = splited_word
                .next()
                .expect("unable to parse span")
                .parse()
                .unwrap();
            return if let Some(context) = col.get_mut(id) {
                context.visited = true;
                data_mode_format(context.val)
            } else {
                let report = miette!(
                    labels = vec![LabeledSpan::at(start..end, "unknown id"),],
                    "Error: Unrecognized identifier {id}"
                );
                errs.push(report);
                "E".to_string()
            };
        }

        word
    }

    let bit_stream: Vec<String> = bit_stream_with_id
        .into_iter()
        .map(|s| handle_id(s, &mut id_collect, &mut errors))
        .collect();

    for (key, context) in id_collect.clone() {
        match context {
            ColType {
                val: _,
                span,
                visited: false,
            } => {
                let report = miette!(
                    severity = Severity::Warning,
                    labels = vec![LabeledSpan::at(span, "Here"),],
                    "Error: {key} declared but never used"
                );
                errors.push(report);
            }
            _ => (),
        }
    }

    ParserReport {
        bit_stream: bit_stream.join(sep),
        report: errors,
        id_collect: id_collect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    #[test]
    fn test_define() {
        let src = "DEFINE foo 0\nDEFINE bar 1\nDEFINE titi 42\nDEFINE tata 73";
        let mut collection: HashMap<String, ColType> = HashMap::new();
        collection.insert(
            "foo".to_string(),
            ColType {
                val: 0,
                span: 7..10,
                visited: false,
            },
        );
        collection.insert(
            "bar".to_string(),
            ColType {
                val: 1,
                span: 20..23,
                visited: false,
            },
        );
        collection.insert(
            "titi".to_string(),
            ColType {
                val: 42,
                span: 33..37,
                visited: false,
            },
        );
        collection.insert(
            "tata".to_string(),
            ColType {
                val: 73,
                span: 48..52,
                visited: false,
            },
        );

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();
        assert_eq!(
            collection,
            parse(&mut tokens, false, false, "").id_collect
        );
        assert_eq!(
            collection,
            parse(&mut tokens, false, true, "").id_collect
        );
    }

    #[test]
    fn test_label() {
        let src = "main:\nJMP\nlabel:\nJMP\nJMP\nJMP\nJMP\nJMP\nJMP\nJMP\nJMP\ntiti:";
        let mut collection: HashMap<String, ColType> = HashMap::new();
        collection.insert(
            "main".to_string(),
            ColType {
                val: 0,
                span: 0..4,
                visited: false,
            },
        );
        collection.insert(
            "label".to_string(),
            ColType {
                val: 16,
                span: 10..15,
                visited: false,
            },
        );
        collection.insert(
            "titi".to_string(),
            ColType {
                val: 144,
                span: 49..53,
                visited: false,
            },
        );

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();
        assert_eq!(
            collection,
            parse(&mut tokens, false, false, "").id_collect
        );
        assert_eq!(
            collection,
            parse(&mut tokens, false, true, "").id_collect
        );
    }

    #[test]
    fn test_load_value() {
        let src = "DEFINE mask 42\nA = 0\nA = 0x7fff\nA = mask";
        let expected = "1000000000000000\n1111111111111111\n1000000000101010";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        assert_eq!(
            expected,
            parse(&mut tokens, false, false, "\n").bit_stream
        );
        assert_eq!(
            expected,
            parse(&mut tokens, false, true, "\n").bit_stream
        );
    }

    #[test]
    fn test_load_value_into_wrong_register() {
        let src = "D = 0";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();
        assert!(!parse(&mut tokens, false, false, "")
            .report
            .is_empty());
    }

    #[test]
    fn test_register_transfer() {
        let src = "D = A\nD = D\nD = *A\nA = D\n*A = D\n*A = A";

        let expected = "0011000000110100\n0011000100110100\n0011000001110100\n0011000100110000\n0011000100110001\n0011000000110001";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        assert_eq!(
            expected,
            parse(&mut tokens, false, false, "\n").bit_stream
        );
        assert_eq!(
            expected,
            parse(&mut tokens, false, true, "\n").bit_stream
        );
    }

    #[test]
    fn test_wrong_register_transfer() {
        let src = "A = *A\nV = *V";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();
        assert!(!parse(&mut tokens, false, false, "")
            .report
            .is_empty())
    }

    #[test]
    fn test_condition() {
        let src = "D==\nD>=\n*A>=";

        let expected = "0010000100110110\n0011000100110110\n0011000001110110";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        assert_eq!(
            expected,
            parse(&mut tokens, false, false, "\n").bit_stream
        );
        assert_eq!(
            expected,
            parse(&mut tokens, false, true, "\n").bit_stream
        );
    }

    #[test]
    fn test_jump() {
        let src = "JMP";

        let expected = "0111000000000000";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        assert_eq!(
            expected,
            parse(&mut tokens, false, false, "\n").bit_stream
        );
        assert_eq!(
            expected,
            parse(&mut tokens, false, true, "\n").bit_stream
        );
    }

    #[test]
    fn test_not() {
        let src = "A = ~D\nD = ~V";

        let expected = "0101000100000000\n0101000010000100";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        assert_eq!(
            expected,
            parse(&mut tokens, false, false, "\n").bit_stream
        );
        assert_eq!(
            expected,
            parse(&mut tokens, false, true, "\n").bit_stream
        );
    }

    #[test]
    #[should_panic]
    fn test_non_single_operand_operation() {
        let src = "A = +D";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        parse(&mut tokens, false, false, "");
    }

    #[test]
    fn test_double_operand_operation() {
        let src = "A = A + D\nA = A & D\nD = *A | A";

        let expected = "0000000000100000\n0010000000100000\n0011000001000100";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        assert_eq!(
            expected,
            parse(&mut tokens, false, false, "\n").bit_stream
        );
        assert_eq!(
            expected,
            parse(&mut tokens, false, true, "\n").bit_stream
        );
    }

    #[test]
    fn test_wrong_double_operand_operation() {
        let src = "A = A ~ D";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        assert!(!parse(&mut tokens, false, false, "")
            .report
            .is_empty());
    }

    #[test]
    fn test_incompatible_registers() {
        let src = "A = A + *A\nV = V + *V\nA = *A & D";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        assert!(!parse(&mut tokens, false, false, "")
            .report
            .is_empty());
    }
}
