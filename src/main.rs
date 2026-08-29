#![warn(clippy::pedantic)]
mod evaluate;
mod expr;
mod interpreter;
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

            // NOTE: The following is ugly:
            // The tokens contains references to inside the scanner, from a mutable borrow
            // That makes it impossible to call lexing_failed(), which would need an immutable borrow
            // Possibly this will be fixed with Polonius
            let _ = scanner.tokenize();

            if matches!(scanner.lexing_failed(), Some(true)) {
                return ExitCode::from(65);
            }

            // Now we need to tokenize again, this time to keep
            let tokens = scanner.tokenize();

            let mut parser = Parser::new(tokens);

            if let Ok(expr) = parser.parse_expression() {
                println!("{expr}");
            } else {
                eprintln!("Parsing failed.");
                return ExitCode::from(65);
            }
        }
        "evaluate" => {
            let mut scanner = Scanner::from_file(filename)
                .unwrap_or_else(|_| panic!("Could not open file {filename}"));

            let mut parser = Parser::new(scanner.tokenize());
            let expr = parser.parse_expression().expect("Parsing failed.");

            match expr.evaluate() {
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
            let mut scanner = Scanner::from_file(filename)
                .unwrap_or_else(|_| panic!("Could not open file {filename}"));

            let mut parser = Parser::new(scanner.tokenize());
            let Ok(program) = parser.parse() else {
                return ExitCode::from(EXIT_CODE_SYNTAX_ERROR);
            };

            let mut interpreter = Interpreter {};

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
