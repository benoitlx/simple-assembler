use crate::lexer::spec::arch_v1::*;
use crate::lexer::{Token::*, *};
use colored::Colorize;
use miette::{miette, Error, LabeledSpan};
use std::collections::HashMap;
use std::ops::Range;

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

/// generate a bit stream from a Vec of Spanned Token
pub fn generate_bit_stream(
    tokens: &mut Vec<(Result<Token, ()>, Range<usize>)>,
    colorize: bool,
    debug: bool,
    sep: &str,
) -> (String, HashMap<String, (u16, Range<usize>)>, Vec<Error>) {
    colored::control::set_override(colorize);
    let mut bit_stream_with_id: Vec<String> = vec![];

    let mut errors: Vec<Error> = vec![];

    // Hashmap for the identifiers
    let mut id_collect: HashMap<String, (u16, Range<usize>)> = HashMap::new();
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
                    id_collect.insert(id.clone(), (adr, span.clone()));
                } else {
                    let other_span = (*id_ref.unwrap()).1.clone();
                    panic!(
                        "identifier already used there {}..{}",
                        other_span.start, other_span.end
                    );
                }

                String::new()
            }
            // DEFINE mask 0x1, tested
            [(Ok(Directive(Dir::Define)), _), (Ok(Identifier(id)), span), (Ok(Value(val)), _), _, _] =>
            {
                i += 3;

                let id_ref = id_collect.get(id);

                if id_ref == None {
                    id_collect.insert(id.clone(), (*val, span.clone()));
                } else {
                    let other_span = (*id_ref.unwrap()).1.clone();
                    panic!(
                        "identifier already used there {}..{}",
                        other_span.start, other_span.end
                    );
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
        col: &mut HashMap<String, (u16, Range<usize>)>,
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
            return if let Some(value) = col.get(id) {
                data_mode_format(value.0)
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

    (bit_stream.join(sep), id_collect, errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    #[test]
    fn test_define() {
        let src = "DEFINE foo 0\nDEFINE bar 1\nDEFINE titi 42\nDEFINE tata 73";
        let mut collection: HashMap<String, (u16, Range<usize>)> = HashMap::new();
        collection.insert("foo".to_string(), (0, 7..10));
        collection.insert("bar".to_string(), (1, 20..23));
        collection.insert("titi".to_string(), (42, 33..37));
        collection.insert("tata".to_string(), (73, 48..52));

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();
        assert_eq!(
            collection,
            generate_bit_stream(&mut tokens, false, false, "").1
        );
        assert_eq!(
            collection,
            generate_bit_stream(&mut tokens, false, true, "").1
        );
        assert_eq!(
            collection,
            generate_bit_stream(&mut tokens, true, false, "").1
        );
        assert_eq!(
            collection,
            generate_bit_stream(&mut tokens, true, true, "").1
        );
    }

    #[test]
    fn test_label() {
        let src = "main:\nJMP\nlabel:\nJMP\nJMP\nJMP\nJMP\nJMP\nJMP\nJMP\nJMP\ntiti:";
        let mut collection: HashMap<String, (u16, Range<usize>)> = HashMap::new();
        collection.insert("main".to_string(), (0, 0..4));
        collection.insert("label".to_string(), (16, 10..15));
        collection.insert("titi".to_string(), (144, 49..53));

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();
        assert_eq!(
            collection,
            generate_bit_stream(&mut tokens, false, false, "").1
        );
        assert_eq!(
            collection,
            generate_bit_stream(&mut tokens, false, true, "").1
        );
        assert_eq!(
            collection,
            generate_bit_stream(&mut tokens, true, false, "").1
        );
        assert_eq!(
            collection,
            generate_bit_stream(&mut tokens, true, true, "").1
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
            generate_bit_stream(&mut tokens, false, false, "\n").0
        );
        assert_eq!(
            expected,
            generate_bit_stream(&mut tokens, false, true, "\n").0
        );
    }

    #[test]
    fn test_load_value_into_wrong_register() {
        let src = "D = 0";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();
        assert!(!generate_bit_stream(&mut tokens, false, false, "")
            .2
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
            generate_bit_stream(&mut tokens, false, false, "\n").0
        );
        assert_eq!(
            expected,
            generate_bit_stream(&mut tokens, false, true, "\n").0
        );
    }

    #[test]
    fn test_wrong_register_transfer() {
        let src = "A = *A\nV = *V";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();
        assert!(!generate_bit_stream(&mut tokens, false, false, "")
            .2
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
            generate_bit_stream(&mut tokens, false, false, "\n").0
        );
        assert_eq!(
            expected,
            generate_bit_stream(&mut tokens, false, true, "\n").0
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
            generate_bit_stream(&mut tokens, false, false, "\n").0
        );
        assert_eq!(
            expected,
            generate_bit_stream(&mut tokens, false, true, "\n").0
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
            generate_bit_stream(&mut tokens, false, false, "\n").0
        );
        assert_eq!(
            expected,
            generate_bit_stream(&mut tokens, false, true, "\n").0
        );
    }

    #[test]
    #[should_panic]
    fn test_non_single_operand_operation() {
        let src = "A = +D";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        generate_bit_stream(&mut tokens, false, false, "");
    }

    #[test]
    fn test_double_operand_operation() {
        let src = "A = A + D\nA = A & D\nD = *A | A";

        let expected = "0000000000100000\n0010000000100000\n0011000001000100";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        assert_eq!(
            expected,
            generate_bit_stream(&mut tokens, false, false, "\n").0
        );
        assert_eq!(
            expected,
            generate_bit_stream(&mut tokens, false, true, "\n").0
        );
    }

    #[test]
    fn test_wrong_double_operand_operation() {
        let src = "A = A ~ D";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        assert!(!generate_bit_stream(&mut tokens, false, false, "")
            .2
            .is_empty());
    }

    #[test]
    fn test_incompatible_registers() {
        let src = "A = A + *A\nV = V + *V\nA = *A & D";

        let lex = Token::lexer(src);

        let mut tokens: Vec<(Result<Token, ()>, std::ops::Range<usize>)> = lex.spanned().collect();

        assert!(!generate_bit_stream(&mut tokens, false, false, "")
            .2
            .is_empty());
    }
}
