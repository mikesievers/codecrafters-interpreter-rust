#![warn(clippy::pedantic)]
mod evaluate;
mod expr;
mod lox_error;
mod lox_value;
mod parser;
mod scanner;
mod stmt;
mod token;

use std::env;
use std::process::ExitCode;

use parser::Parser;
pub use scanner::Scanner;

use crate::evaluate::Evaluate;
use crate::lox_error::LoxError;
use crate::stmt::Stmt;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} (tokenize|parse) <filename>", args[0]);
        return ExitCode::FAILURE;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            // eprintln!("Logs from your program will appear here!");

            let mut scanner = Scanner::from_file(filename)
                .unwrap_or_else(|_| panic!("Could not open file {filename}"));

            for token in scanner.tokenize() {
                println!("{token}");
            }
            if matches!(scanner.lexing_failed(), Some(true)) {
                return ExitCode::from(65);
            }
        }
        "parse" => {
            let mut scanner = Scanner::from_file(filename)
                .unwrap_or_else(|_| panic!("Could not open file {filename}"));

            let mut parser = Parser::new(scanner.tokenize());

            if let Ok(program) = parser.parse() {
                for stmt in program {
                    println!("{stmt}");
                }
            } else {
                eprintln!("Parsing failed.");
                return ExitCode::from(65);
            }
        }
        "evaluate" => {
            let mut scanner = Scanner::from_file(filename)
                .unwrap_or_else(|_| panic!("Could not open file {filename}"));

            let mut parser = Parser::new(scanner.tokenize());
            let program = parser.parse().expect("Parsing failed.");

            if let Some(Stmt::Expression(expr)) = program.first() {
                // The first line of the program is not an expression
                match expr.evaluate() {
                    Ok(output) => println!("{output}"),
                    Err(LoxError::RuntimeError(e)) => {
                        eprintln!("{e}");
                        return ExitCode::from(70);
                    }
                    Err(LoxError::SyntaxError(e)) => {
                        eprintln!("{e}");
                        return ExitCode::from(65);
                    }
                }
            } else {
                return ExitCode::from(1);
            }
        }
        "run" => {
            let mut scanner = Scanner::from_file(filename)
                .unwrap_or_else(|_| panic!("Could not open file {filename}"));

            let mut parser = Parser::new(scanner.tokenize());
            let program = parser.parse().expect("Parsing failed.");

            for stmt in program {
                match stmt.execute() {
                    Ok(()) => (),
                    Err(LoxError::RuntimeError(e)) => {
                        eprintln!("{e}");
                        return ExitCode::from(70);
                    }
                    Err(LoxError::SyntaxError(e)) => {
                        eprintln!("{e}");
                        return ExitCode::from(65);
                    }
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {command}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
