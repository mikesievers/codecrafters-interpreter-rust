#![allow(unused_variables)]
mod scanner;
mod token;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use std::env;

pub use scanner::Scanner;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} tokenize <filename>", args[0]);
        return Ok(());
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            // eprintln!("Logs from your program will appear here!");

            let scanner = Scanner::from_file(filename)
                .with_context(|| format!("Could not open file {filename}"))?;

            for token in scanner.tokenize() {
                println!("{token}");
            }
        }
        _ => {
            bail!("Unknown command: {}", command);
        }
    }
    Ok(())
}
