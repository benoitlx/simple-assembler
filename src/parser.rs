use crate::lexer::spec::arch_v1::*;
use crate::lexer::{Token::*, *};
use colored::Colorize;
use logos::Lexer;
use std::collections::HashMap;

/*
TODO:
- parser les instructions avec un non ~
- parser les instructions du types A = D (avec lex.peek())
- tests
- soigner le code
- pour chaque panic donner les bonnes infos l'endroit du token fautif ...

Futur:
- Error handling with miette
- cli
*/

pub fn generate_bit_stream(lex: &mut Lexer<Token>) -> String {
    let mut bit_stream = String::new();

    // Hashmap for the identifiers
    let mut id_collect: HashMap<String, (u16, usize, usize)> = HashMap::new();
    let mut adr = 0;

    // let mut lex = lex.peekable();

    while let Some(Ok(token)) = lex.next() {
        let current_bit_stream: Option<String> = match token {
            Directive(Dir::Define) => {
                match lex.next() {
                    Some(Ok(Identifier(id))) => {
                        let start = lex.span().start;
                        let end = lex.span().end;
                        match lex.next() {
                            Some(Ok(Value(v))) => {
                                let id_ref = id_collect.get(&id);

                                if id_ref == None {
                                    id_collect.insert(id, (v, start, end));
                                } else {
                                    let start = (*id_ref.unwrap()).1;
                                    let end = (*id_ref.unwrap()).2;
                                    panic!("identifier already used there {}..{}", start, end);
                                }
                            }
                            Some(Ok(t)) => panic!("Expected Token::Identifier, found {:?}", t),
                            _ => panic!("Found EOF or unknown token, expected Token::Identifier"),
                        }
                    }
                    Some(Ok(t)) => panic!("Expected Token::Identifier, found {:?}", t),
                    _ => panic!("Found EOF or unknown token, expected Token::Identifier"),
                }
                None
            }
            Identifier(id) => {
                // todo: replace the code below with a match statement
                if let Some(Ok(token)) = lex.next() {
                    if token != Directive(Dir::Label) {
                        panic!("Expected Dir::Label found {:?}", token);
                    } else {
                        let id_ref = id_collect.get(&id);

                        if id_ref == None {
                            id_collect.insert(id, (adr, lex.span().start, lex.span().end));
                        } else {
                            let start = (*id_ref.unwrap()).1;
                            let end = (*id_ref.unwrap()).2;
                            panic!("identifier already used there {}..{}", start, end);
                        }
                    }
                } else {
                    panic!("Expected Dir::Label found EOF");
                }
                None
            }
            Condition(cond) if cond == Cond::Jump => {
                adr += 16;
                Some(format!(
                    "{}{}000000000000",
                    "1".green().bold(),
                    cond.bit_stream().blue()
                ))
            }
            Register(regc) => {
                adr += 16;
                let mut inst_bits = String::new();
                let mut bits_a = String::new();
                let mut bits_b = String::new();
                let bits_c = regc.bit_stream();
                let mut bits_op = String::new();

                let mut inst_mode = false;

                match lex.next() {
                    Some(Ok(Token::Assignement)) => (),
                    Some(Ok(t)) => panic!("Expected Token::Assignement, found {:?}", t),
                    _ => panic!("Found EOF or unknown token, expected Token::Assignement"),
                }

                match lex.next() {
                    Some(Ok(Value(integer))) => {
                        inst_bits.push('1');
                        inst_bits.push_str(format!("{:b}", integer).as_str());
                        break; // all tokens are consumed for this instruction
                    }
                    Some(Ok(Identifier(id))) => {
                        inst_bits.push('1');
                        inst_bits.push_str(id.as_str());
                    }
                    Some(Ok(Register(rega))) => {
                        inst_mode = true;
                        bits_a = rega.bit_stream();
                    }
                    Some(Ok(t)) => panic!("Found {:?}, expected one of Token::Value(_) or Token::Register(_)", t),
                    _ => panic!("Found EOF or unknown token, expected one of Token::Value(_) or Token::Register(_)")
                }

                if inst_mode {
                    match lex.next() {
                        Some(Ok(Operation(op))) => {
                            bits_op = op.bit_stream();
                        }
                        Some(Ok(t)) => panic!(
                            "Expected Token::Operation, found {:?} {}..{}",
                            t,
                            lex.span().start,
                            lex.span().end
                        ),
                        _ => panic!("Found EOF or unknown token, expected Token::Operation"),
                    }

                    match lex.next() {
                        Some(Ok(Register(regb))) => {
                            bits_b = regb.bit_stream();
                        }
                        Some(Ok(t)) => panic!("Expected Token::Register, found {:?}", t),
                        _ => panic!("Found EOF or unknown token, expected Token::Register"),
                    }
                    inst_bits = format!(
                        "{}{}{}{}{}000",
                        "0".green().bold(),
                        bits_op.blue(),
                        bits_a.yellow(),
                        bits_b.purple(),
                        bits_c.cyan()
                    );
                }

                Some(inst_bits)
            }
            Comment => None,
            _ => panic!(
                "Can't start with something other than label directive register or jump {}..{}",
                lex.span().start,
                lex.span().end
            ),
        };

        if let Some(mut str_to_push) = current_bit_stream {
            str_to_push.push('\n');
            bit_stream.push_str(&str_to_push.as_str());
        }
    }

    println!(
        "{}\n{}\n{}\n{}\n{}\n{}\n",
        "15 bits value".red(),
        "op/jump code".blue(),
        "mode bit".green(),
        "source A reg".yellow(),
        "source B reg".purple(),
        "dest reg".cyan()
    );

    replace_identifiers(&bit_stream, &id_collect)
}

fn replace_identifiers(input: &str, id_collect: &HashMap<String, (u16, usize, usize)>) -> String {
    let mut output = String::new();
    let mut i = 0;
    let chars: Vec<char> = input.chars().collect();

    while i < chars.len() {
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphabetic() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if let Some(value) = id_collect.get(&word) {
                output.push_str(&format!("{}", format!("{:015b}", value.0).red()));
            } else {
                output.push_str(&word);
            }
        } else {
            output.push(chars[i]);
            i += 1;
        }
    }

    output
}
