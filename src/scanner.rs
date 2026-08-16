use std::fs::read_to_string;

use anyhow::Result;

use crate::token::{Token, TokenType};

pub struct Scanner {
    data: String,
    lexical_errors_found: Option<bool>,
}

impl<'a> Scanner {
    pub fn from_string(data: String) -> Self {
        Scanner {
            data,
            lexical_errors_found: None,
        }
    }

    pub fn from_file(filename: &String) -> Result<Self> {
        let data = read_to_string(filename)?;
        Ok(Scanner {
            data,
            lexical_errors_found: None,
        })
    }

    // The lexemes are references into self.data, therefore
    // Self must live as long as the output Tokens are used
    pub fn tokenize(&'a mut self) -> Vec<Token<'a>> {
        let line_no: u32 = 1;
        let mut tokens: Vec<Token<'a>> = vec![];
        self.lexical_errors_found = Some(false);

        let mut char_indices = self.data.char_indices().peekable();

        loop {
            match char_indices.next() {
                Some((byte_idx, c))
                    if let Some(token) =
                        token_from_single_char(&self.data[byte_idx..byte_idx + c.len_utf8()]) =>
                {
                    tokens.push(token)
                }
                // default: emit error message
                Some((byte_idx, c)) => {
                    eprintln!("[line {}] Error: Unexpected character: {}", line_no, c);
                    self.lexical_errors_found = Some(true);
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

    pub fn lexing_failed(&self) -> Option<bool> {
        self.lexical_errors_found
    }
}

pub fn token_from_single_char<'a>(c: &'a str) -> Option<Token<'a>> {
    match c {
        "(" => Some(Token {
            token_type: TokenType::LeftParen,
            lexeme: c,
            literal: None,
        }),
        ")" => Some(Token {
            token_type: TokenType::RightParen,
            lexeme: c,
            literal: None,
        }),
        "{" => Some(Token {
            token_type: TokenType::LeftBrace,
            lexeme: c,
            literal: None,
        }),
        "}" => Some(Token {
            token_type: TokenType::RightBrace,
            lexeme: c,
            literal: None,
        }),
        "," => Some(Token {
            token_type: TokenType::Comma,
            lexeme: c,
            literal: None,
        }),
        "." => Some(Token {
            token_type: TokenType::Dot,
            lexeme: c,
            literal: None,
        }),
        "-" => Some(Token {
            token_type: TokenType::Minus,
            lexeme: c,
            literal: None,
        }),
        "+" => Some(Token {
            token_type: TokenType::Plus,
            lexeme: c,
            literal: None,
        }),
        "*" => Some(Token {
            token_type: TokenType::Star,
            lexeme: c,
            literal: None,
        }),
        ";" => Some(Token {
            token_type: TokenType::Semicolon,
            lexeme: c,
            literal: None,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use itertools::Itertools;

    #[test]
    fn test_scanner() {
        let mut scanner = Scanner::from_string("()".into());

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
