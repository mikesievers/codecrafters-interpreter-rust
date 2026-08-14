use std::fs::read_to_string;

use anyhow::Result;

use crate::token::Token;

pub struct Scanner {
    data: String,
}

impl Scanner {
    pub fn from_string(data: String) -> Self {
        Scanner { data }
    }

    pub fn from_file(filename: &String) -> Result<Self> {
        let data = read_to_string(filename)?;
        Ok(Scanner { data })
    }

    pub fn tokenize(&self) -> Vec<Token> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Display;

    use itertools::Itertools;

    #[test]
    fn test_scanner() {
        let scanner = Scanner::from_string("()".into());

        // assert_eq!(
        //     scanner
        //         .tokenize()
        //         .iter()
        //         .map(|token| { token.fmt() })
        //         .collect_vec(),
        //     vec!["LEFT_PAREN ( null", "RIGHT_PAREN ) null", "EOF  null"]
        // );
    }
}
