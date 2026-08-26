use crate::Instruction::{A, C};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::str::Chars;
use std::time::Instant;

fn read_file(filename: &str) -> String {
    fs::read_to_string(filename).expect("Failed to read the file")
}

fn write_file(filename: &str, text: &str) {
    fs::write(filename, text).expect("Failed to write file");
}

#[derive(Debug)]
enum Instruction {
    A(String),
    C {
        dest: Option<String>,
        comp: String,
        jump: Option<String>,
    },
}

fn get_command(itr: &mut Chars, first: Option<char>) -> String {
    let mut next = itr.next();
    let mut command = match first {
        None => String::new(),
        Some(c) => String::from(c),
    };
    let mut ignore = false;
    while let Some(c) = next
        && c != '\n'
    {
        if c == '/' {
            next = itr.next();
            if next.is_none() || !next.unwrap().eq(&'/') {
                command.push('/');
                continue;
            }
            ignore = true;
        }
        if !ignore {
            match c {
                ' ' => {}
                '\t' => {}
                '\r' => {}
                _ => {
                    command.push(c);
                }
            }
        }
        next = itr.next();
    }
    command
}

fn first_pass(input: &str, symbols: &mut HashMap<String, u16>) -> Vec<Instruction> {
    let mut instructions: Vec<Instruction> = Vec::new();
    let mut itr = input.chars().into_iter();
    let mut next = itr.next();
    let mut index: u16 = 0;
    while next.is_some() {
        match next.unwrap() {
            ' ' => {
                next = itr.next();
            }
            '\n' => {
                next = itr.next();
            }
            '\t' => {
                next = itr.next();
            }
            '\r' => {
                next = itr.next();
            }
            '/' => {
                while next.is_some() && !next.unwrap().eq(&'\n') {
                    next = itr.next();
                }
                next = itr.next();
            }
            '(' => {
                next = itr.next();
                let mut name = String::new();
                while !next.expect("Syntax error").eq(&')') {
                    name.push(next.unwrap());
                    next = itr.next();
                }
                symbols.insert(name, index);
                next = itr.next();
            }
            '@' => {
                instructions.push(A(get_command(&mut itr, None)));
                next = itr.next();
                index = index + 1;
            }
            _ => {
                let command = get_command(&mut itr, next.clone());
                let (dest, rest) = match command.split_once("=") {
                    None => (None, command.as_str()),
                    Some((dest, rest)) => (Some(dest.to_string()), rest),
                };
                let (comp, jump) = match rest.split_once(";") {
                    None => (rest.to_string(), None),
                    Some((c, j)) => (c.to_string(), Some(j.to_string())),
                };
                instructions.push(C { dest, comp, jump });
                next = itr.next();
                index += 1;
            }
        }
    }

    instructions
}

fn second_pass(instructions: &Vec<Instruction>, symbols: &mut HashMap<String, u16>) -> String {
    let mut binary_code = String::new();
    let mut memory_index: u16 = 15;
    for instruction in instructions {
        match instruction {
            A(command) => {
                binary_code.push('0');
                binary_code.push_str(
                    format!(
                        "{:015b}",
                        if let Some(number) = command
                            .parse::<u16>()
                            .ok()
                            .or_else(|| { symbols.get(command).cloned() })
                        {
                            number
                        } else {
                            memory_index += 1;
                            symbols.insert(command.to_string(), memory_index);
                            memory_index.clone()
                        }
                    )
                    .as_str(),
                );
            }

            C { dest, comp, jump } => {
                binary_code.push_str("111");
                binary_code.push_str(match comp.as_str() {
                    "0" => "0101010",
                    "1" => "0111111",
                    "-1" => "0111010",
                    "D" => "0001100",
                    "A" => "0110000",
                    "!D" => "0001101",
                    "!A" => "0110001",
                    "-D" => "0001111",
                    "-A" => "0110011",
                    "D+1" => "0011111",
                    "A+1" => "0110111",
                    "D-1" => "0001110",
                    "A-1" => "0110010",
                    "D+A" => "0000010",
                    "D-A" => "0010011",
                    "A-D" => "0000111",
                    "D&A" => "0000000",
                    "D|A" => "0010101",
                    "M" => "1110000",
                    "!M" => "1110001",
                    "-M" => "1110011",
                    "M+1" => "1110111",
                    "M-1" => "1110010",
                    "D+M" => "1000010",
                    "D-M" => "1010011",
                    "M-D" => "1000111",
                    "D&M" => "1000000",
                    "D|M" => "1010101",
                    _ => {
                        panic!(
                            "panic!!!! what are you doing?!?! there is a syntax error!! {} ",
                            comp.as_str()
                        )
                    }
                });

                binary_code.push_str(match dest {
                    None => "000",
                    Some(s) => match s.as_str() {
                        "" => "000",
                        "M" => "001",
                        "D" => "010",
                        "MD" => "011",
                        "A" => "100",
                        "AM" => "101",
                        "AD" => "110",
                        "AMD" => "111",
                        _ => {
                            panic!("panic!!!! what are you doing?!?! there is a syntax error!!")
                        }
                    },
                });
                binary_code.push_str(match jump {
                    None => "000",
                    Some(s) => match s.as_str() {
                        "" => "000",
                        "JGT" => "001",
                        "JEQ" => "010",
                        "JGE" => "011",
                        "JLT" => "100",
                        "JNE" => "101",
                        "JLE" => "110",
                        "JMP" => "111",
                        _ => {
                            panic!("panic!!!! what are you doing?!?! there is a syntax error!!")
                        }
                    },
                });
            }
        }
        binary_code.push('\n')
    }
    binary_code
}

fn get_hash_map() -> HashMap<String, u16> {
    let mut hash_map = HashMap::new();
    hash_map.insert(String::from("SP"), 0);
    hash_map.insert(String::from("LCL"), 1);
    hash_map.insert(String::from("ARG"), 2);
    hash_map.insert(String::from("THIS"), 3);
    hash_map.insert(String::from("THAT"), 4);
    hash_map.insert(String::from("SCREEN"), 16384);
    hash_map.insert(String::from("KBD"), 24576);
    for i in 0..15 {
        hash_map.insert(format!("R{}", i), i);
    }
    hash_map
}

fn parse_args() -> Option<(String, String)> {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        return None;
    }
    let mut op = 0;
    let mut filename = String::new();
    let mut output_file = String::new();
    for arg in args.into_iter() {
        if op == 0 {
            op += 1;
            continue;
        }
        if arg.starts_with("-") {
            op = match arg.as_str() {
                "-o" => 2,
                "-f" => 1,
                _ => return None,
            }
        } else {
            match op {
                1 => {
                    filename = arg;
                    op += 1
                }
                2 => output_file = arg,
                _ => break,
            }
        }
    }
    if output_file.is_empty() {
        output_file = format!("{}.hack", filename.split(".").next().unwrap().to_string());
    }
    Some((filename, output_file))
}

fn main() {
    let start = Instant::now();
    let (file_name, output_file) = match parse_args() {
        None => {
            println!("Usage: hack_assembler [-f]<filename> [-o]<output_file>");
            return;
        }
        Some(stuff) => stuff,
    };
    let content = read_file(&file_name);
    println!("successfully read {}.", file_name);
    let mut symbols = get_hash_map();
    let instructions = first_pass(&content, &mut symbols);
    let binary_code = second_pass(&instructions, &mut symbols);
    println!("assembled successfully.");
    write_file(&output_file, &binary_code);
    println!("successfully wrote to {}.", output_file);
    let duration = start.elapsed();
    println!("Done! took {:?}ms", duration.as_millis());
}
