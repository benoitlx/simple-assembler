use crate::lexer::spec::arch_v1::*;
use crate::lexer::{Token::*, *};
use colored::Colorize;
use std::collections::HashMap;
use std::ops::Range;

/*
TODO:
- should merge Op and Cond since they have the same bit placement in the instruction
- Add default implementation for Reg Cond and Op
- parser les conditions
- tests
- pour chaque panic donner les bonnes infos l'endroit du token fautif ...

Futur:
- Error handling with miette
- cli
*/

fn data_mode_format(val: u16) -> String {
    format!("{}{}", "1".green(), format!("{:015b}", val).red())
}

fn inst_mode_format(op_or_cond: Op, rega: Reg, regb: Reg, regc: Reg) -> String {
    format!(
        "{}{}{}{}{}000",
        "0".green().bold(),
        op_or_cond.bit_stream().blue(),
        rega.bit_stream().yellow(),
        regb.bit_stream().purple(),
        regc.bit_stream().cyan()
    )
}

pub fn generate_bit_stream_v2(
    tokens: &mut Vec<(Result<Token, ()>, Range<usize>)>,
    colorize: bool,
    debug: bool,
    sep: &str,
) -> String {
    colored::control::set_override(colorize);
    if colorize {
        println!(
            "{}\n{}\n{}\n{}\n{}\n{}\n",
            "15 bits value".red(),
            "op/jump code".blue(),
            "mode bit".green(),
            "source A reg".yellow(),
            "source B reg".purple(),
            "dest reg".cyan()
        );
    }

    let mut bit_stream_with_id: Vec<String> = vec![];

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
            [(Ok(Register(regc)), _), (Ok(Assignement), _), (Ok(Register(rega)), _), (Ok(Operation(op)), _), (Ok(Register(regb)), _)] =>
            {
                // FIXME: protection rule between the regs
                i += 5;
                adr += 16;
                inst_mode_format(*op, *rega, *regb, *regc)
            }
            // A <- mask
            [(Ok(Register(regc)), _), (Ok(Assignement), _), (Ok(Identifier(id)), _), _, _] => {
                if *regc != Reg::A {
                    panic!("Can't push direct value into an other register than A")
                }

                i += 3;
                adr += 16;
                id.clone()
            }
            // A <- 0x7fff
            [(Ok(Register(regc)), _), (Ok(Assignement), _), (Ok(Value(val)), _), _, _] => {
                if *regc != Reg::A {
                    panic!("Can't push direct value into an other register than A")
                }

                i += 3;
                adr += 16;
                data_mode_format(*val)
            }
            // A <- D
            [(Ok(Register(regc)), _), (Ok(Assignement), _), (Ok(Register(rega)), _), _, _] => {
                i += 3;
                adr += 16;
                inst_mode_format(Op::Or, *rega, Reg::A, *regc) // Fixme: add Reg::One and Reg::Zero
            }
            // A <- ~D
            [(Ok(Register(regc)), _), (Ok(Assignement), _), (Ok(Operation(op)), _), (Ok(Register(rega)), _), _] =>
            {
                i += 4;
                adr += 16;
                inst_mode_format(*op, *rega, Reg::A, *regc) // Fixme: add Reg::One and Reg::Zero
            }
            // A == D
            [(Ok(Register(rega)), _), (Ok(Condition(_cond)), _), (Ok(Register(regb)), _), _, _] => {
                i += 1;
                adr += 16;
                inst_mode_format(Op::Add, *rega, *regb, Reg::A) // Fixme: argument type for cond
            }
            // JMP
            [(Ok(Condition(Cond::Jump)), _), _, _, _, _] => {
                i += 1;
                adr += 16;
                // inst_mode_format(Op::Add, Reg::A, Reg::A, Reg::A) // fixme: replace with proper values
                format!("0111000000000000")
            }
            // label:
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
            // DEFINE mask 0x1
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

    fn handle_id(id: String, col: &mut HashMap<String, (u16, Range<usize>)>) -> String {
        if id.chars().all(|c| c.is_alphabetic() || c == '_') {
            return if let Some(value) = col.get(&id) {
                data_mode_format(value.0)
            } else {
                "Error".to_string()
                // todo return a proper error and where it happened
            };
        }

        id
    }

    let bit_stream: Vec<String> = bit_stream_with_id
        .into_iter()
        .map(|s| handle_id(s, &mut id_collect))
        .collect();

    bit_stream.join(sep)
}
