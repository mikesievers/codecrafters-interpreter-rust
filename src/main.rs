#![allow(unused_variables)]
mod scanner;
mod token;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use std::env;
use std::fs;

use scanner::Scanner;
use token::Token;

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
            eprintln!("Logs from your program will appear here!");

            let scanner = Scanner::from_file(filename)
                .with_context(|| format!("Could not open file {filename}"))?;

            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            if !file_contents.is_empty() {
                bail!("Scanner not implemented");
            } else {
                println!("EOF  null"); // Placeholder, replace this line when implementing the scanner
            }
        }
        _ => {
            bail!("Unknown command: {}", command);
        }
    }
    Ok(())
}
