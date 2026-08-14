#![allow(unused_variables)]
mod scanner;
mod token;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use std::env;
use std::process::ExitCode;

pub use scanner::Scanner;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} tokenize <filename>", args[0]);
        return ExitCode::FAILURE;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            // eprintln!("Logs from your program will appear here!");

            let mut scanner = Scanner::from_file(filename)
                .expect(format!("Could not open file {}", filename).as_str());

            for token in scanner.tokenize() {
                println!("{token}");
            }
            if matches!(scanner.lexing_failed(), Some(true)) {
                return ExitCode::from(65);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
