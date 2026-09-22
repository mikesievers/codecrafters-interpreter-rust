#![warn(clippy::pedantic)]
mod environment;
mod expr;
mod interpreter;
mod lox_error;
mod lox_function;
mod lox_value;
mod parser;
mod scanner;
mod stmt;
mod token;

use std::env;
use std::process::ExitCode;

use parser::Parser;
pub use scanner::Scanner;

use crate::interpreter::Interpreter;
use crate::lox_error::LoxError;

const EXIT_CODE_SYNTAX_ERROR: u8 = 65;
const EXIT_CODE_RUNTIME_ERROR: u8 = 70;

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

            let scanner = Scanner::from_file(filename)
                .unwrap_or_else(|_| panic!("Could not open file {filename}"));

            let (tokens, had_errors) = scanner.tokenize();
            for token in &tokens {
                println!("{token}");
            }
            if had_errors {
                return ExitCode::from(65);
            }
        }
        "parse" => {
            let scanner = Scanner::from_file(filename)
                .unwrap_or_else(|_| panic!("Could not open file {filename}"));

            let (tokens, had_errors) = scanner.tokenize();
            if had_errors {
                return ExitCode::from(65);
            }

            let mut parser = Parser::new(tokens);

            if let Ok(expr) = parser.parse_expression() {
                println!("{expr}");
            } else {
                eprintln!("Parsing failed.");
                return ExitCode::from(65);
            }
        }
        "evaluate" => {
            let scanner = Scanner::from_file(filename)
                .unwrap_or_else(|_| panic!("Could not open file {filename}"));

            let (tokens, _) = scanner.tokenize();
            let mut parser = Parser::new(tokens);
            let expr = parser.parse_expression().expect("Parsing failed.");

            let mut interpreter = Interpreter::new();

            match interpreter.evaluate(&expr) {
                Ok(output) => println!("{output}"),
                Err(LoxError::RuntimeError(e)) => {
                    eprintln!("{e}");
                    return ExitCode::from(EXIT_CODE_RUNTIME_ERROR);
                }
                Err(LoxError::SyntaxError(e)) => {
                    eprintln!("{e}");
                    return ExitCode::from(EXIT_CODE_SYNTAX_ERROR);
                }
            }
        }
        "run" => {
            let scanner = Scanner::from_file(filename)
                .unwrap_or_else(|_| panic!("Could not open file {filename}"));

            let (tokens, had_errors) = scanner.tokenize();
            if had_errors {
                eprintln!("Token scanning produced errors, not continuing.");
                return ExitCode::from(EXIT_CODE_SYNTAX_ERROR);
            }

            let mut parser = Parser::new(tokens);
            let program = match parser.parse() {
                Ok(program) => program,
                Err(e) => {
                    eprintln!("Parsing failed: {e}");
                    return ExitCode::from(EXIT_CODE_SYNTAX_ERROR);
                }
            };

            let mut interpreter = Interpreter::new();

            match interpreter.interpret(&program) {
                Ok(()) => (),
                Err(LoxError::RuntimeError(_)) => {
                    return ExitCode::from(EXIT_CODE_RUNTIME_ERROR);
                }
                Err(LoxError::SyntaxError(_)) => {
                    return ExitCode::from(EXIT_CODE_SYNTAX_ERROR);
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
