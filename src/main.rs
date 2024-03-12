mod emitter;
mod ast;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub lexparse);

use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};

use emitter::*;
use emitter::error::CompilerError;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("Please supply exactly 2 arguments.");
    }

    let input_file_path = args[1].clone();
    let output_file_path = args[2].clone();
    let compilee = fs::read_to_string(&input_file_path)
        .expect("Failed to read input file");

    match lexparse::ProgramParser::new().parse(&compilee) {
        Ok(ast) => {
            if let Ok(mut pseudo_assembler) = Emitter::new(ast) {
                if let Ok(_) = pseudo_assembler.construct() {
                    let ass = pseudo_assembler.emit();
                    fs::write(&output_file_path, ass)
                        .expect("Unable to write to file");
                } else {
                    eprintln!("Error in constructing pseudo assembler");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error in Emitter initialization");
                std::process::exit(1);
            }
        },
        Err(_) => println!("Syntax Error"),
    };
}
#[warn(dead_code)]
fn write_message_and_exit(error: CompilerError, input_file_path: &str) {
    let line_no = find_line_number(input_file_path, error.get_byte())
        .unwrap_or_else(|| {
            eprintln!("Unable to find line number");
            std::process::exit(1);
        });

    let error_message = match error {
        CompilerError::UndeclaredVariable(id, _) |
        CompilerError::IncorrectUseOfVariable(id, _) |
        CompilerError::IndexOutOfBounds(id, _) |
        CompilerError::ArrayUsedAsIndex(id, _) |
        CompilerError::WrongArgumentType(id, _) |
        CompilerError::DuplicateVariableDeclaration(id, _) => {
            format!("ERROR: `{}` line: {}", id.split('@').next().unwrap(), line_no)
        },
        _ => format!("ERROR: {:?} line: {}", error, line_no),
    };

    println!("{}", error_message);
    std::process::exit(1);
}

fn find_line_number(file_path: &str, byte_index: usize) -> Option<usize> {
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    let mut total_bytes = 0;
    for (i, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        total_bytes += line.len() + 1; // +1 for the '\n' character
        if total_bytes >= byte_index {
            return Some(i + 1); // +1 because line numbers start from 1
        }
    }
    None
}
