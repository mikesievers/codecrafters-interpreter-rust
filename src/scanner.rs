use std::fs::read_to_string;

use anyhow::Result;
use itertools::Itertools;

use crate::token::Token;

pub struct Scanner {
    data: String,
}

impl<'a> Scanner {
    pub fn from_string(data: String) -> Self {
        Scanner { data }
    }

    pub fn from_file(filename: &String) -> Result<Self> {
        let data = read_to_string(filename)?;
        Ok(Scanner { data })
    }

    // The lexemes are references into self.data, therefore
    // Self must live as long as the output Tokens are used
    pub fn tokenize(&'a self) -> Vec<Token<'a>> {
        // let iter = self.data.chars().peekable();

        let mut line_no: u32 = 1;
        let mut tokens: Vec<Token<'a>> = vec![];

        let mut chars = self.data.chars().peekable();

        loop {
            match chars.next() {
                Some(c) => todo!(),
                None => tokens.push(Token::eof()),
            }
        }

        // self.data
        //     .chars()
        //     .peekable()
        //     .filter_map(
        //         |c| match c {
        //             _ => {
        //                 eprintln!("Unknwn character in line {}: {}", line_no, c);
        //                 None
        //             }
        //         }, // Token {
        //            // token_type: todo!(),
        //            // lexeme: todo!(),
        //            // literal: todo!(),
        //            // }
        //     )
        //     .collect_vec()

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use itertools::Itertools;

    #[test]
    fn test_scanner() {
        let scanner = Scanner::from_string("()".into());

        assert_eq!(
            scanner
                .tokenize()
                .iter()
                .map(|token| { token.display() })
                .collect_vec(),
            vec!["LEFT_PAREN ( null", "RIGHT_PAREN ) null", "EOF  null"]
        );
    }
}
