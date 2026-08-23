#![warn(clippy::pedantic)]
mod expr;
mod parser;
mod scanner;
mod token;
mod lox_value;
mod evaluate;

use std::env;
use std::process::ExitCode;

use parser::Parser;
pub use scanner::Scanner;

use crate::evaluate::Evaluate;

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

            if let Ok(expr) = parser.parse() { println!("{expr}") } else { 
                eprintln!("Parsing failed."); 
                return ExitCode::from(65);
            }

        }
        "evaluate" => {
            let mut scanner = Scanner::from_file(filename)
                .unwrap_or_else(|_| panic!("Could not open file {filename}"));

            let mut parser = Parser::new(scanner.tokenize());

            if let Ok(expr) = parser.parse() { println!("{}", expr.evaluate()) } else { 
                eprintln!("Parsing failed."); 
                return ExitCode::from(65);
            }
        }

        _ => {
            eprintln!("Unknown command: {command}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
