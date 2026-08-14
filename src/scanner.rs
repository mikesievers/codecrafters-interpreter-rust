use std::fs::read_to_string;

use anyhow::Result;

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

        let mut char_indices = self.data.char_indices().peekable();

        loop {
            match char_indices.next() {
                Some((byte_idx, c))
                    if let Some(token) = Token::token_from_single_char(
                        &self.data[byte_idx..byte_idx + c.len_utf8()],
                    ) =>
                {
                    tokens.push(token)
                }
                // default: emit error message
                Some((byte_idx, c)) => {
                    eprintln!("Unkown character on line {}: {}", line_no, c);
                }
                // No more chars -> EOF and break
                None => {
                    tokens.push(Token::eof());
                    break;
                }
            }
        }

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
