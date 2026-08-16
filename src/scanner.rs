use std::fs::read_to_string;

use anyhow::Result;
use itertools::peek_nth;

use crate::token::{Token, TokenType, TokenValue};

pub struct Scanner {
    data: String,
    lexical_errors_found: Option<bool>,
}

impl Scanner {
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
    // the returned tokens borrow from self while they are in use.
    pub fn tokenize(&mut self) -> Vec<Token<'_>> {
        let mut line_no: u32 = 1;
        let mut tokens: Vec<Token<'_>> = vec![];
        self.lexical_errors_found = Some(false);

        let mut char_indices = peek_nth(self.data.char_indices());

        loop {
            match char_indices.next() {
                Some((byte_idx, c))
                    if let Some(token) =
                        token_from_single_char(&self.data[byte_idx..byte_idx + c.len_utf8()]) =>
                {
                    tokens.push(token)
                }
                // '=' Equality / assignment
                Some((byte_idx, '=')) => {
                    handle_equal(&self.data, &mut tokens, &mut char_indices, byte_idx);
                }
                // '!' Bang / inequality
                Some((byte_idx, '!')) => {
                    handle_bang(&self.data, &mut tokens, &mut char_indices, byte_idx);
                }
                // '<' Less / LEQ
                Some((byte_idx, '<')) => {
                    handle_less(&self.data, &mut tokens, &mut char_indices, byte_idx);
                }
                // '>' Greater / GEQ
                Some((byte_idx, '>')) => {
                    handle_greater(&self.data, &mut tokens, &mut char_indices, byte_idx);
                }
                // '/' Slash/comment
                Some((byte_idx, '/')) => {
                    handle_slash(&self.data, &mut tokens, &mut char_indices, byte_idx);
                }
                // Whitespace (Tab, Space, New Line)
                Some((byte_idx, c)) if c == ' ' || c == '\t' || c == '\n' => {
                    if c == '\n' {
                        line_no += 1;
                    }
                }
                // '"' String
                Some((byte_idx, '"')) => {
                    match handle_string(&self.data, &mut tokens, &mut char_indices, byte_idx) {
                        Some(n) => line_no += n,
                        None => {
                            eprintln!("[line {}] Error: Unterminated string.", line_no);
                            self.lexical_errors_found = Some(true);
                        }
                    }
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

fn handle_string<'a>(
    data: &'a str,
    tokens: &mut Vec<Token<'a>>,
    char_indices: &mut itertools::PeekNth<std::str::CharIndices<'a>>,
    byte_idx: usize,
) -> Option<u32> {
    // String: consume everything until the next double quote
    // Return None on EOF
    let quote_len = '"'.len_utf8();
    let mut byte_len = quote_len;
    let mut new_lines = 0;

    loop {
        if let Some((byte_idx_next, c_next)) = char_indices.next() {
            byte_len += c_next.len_utf8();
            if c_next == '"' {
                break;
            }
            if c_next == '\n' {
                new_lines += 1;
            }
        } else {
            return None;
        }
    }

    tokens.push(Token {
        token_type: TokenType::String,
        lexeme: &data[byte_idx..byte_idx + byte_len],
        literal: Some(TokenValue::String(
            &data[byte_idx + quote_len..byte_idx + byte_len - quote_len],
        )),
    });
    Some(new_lines)
}

fn handle_slash<'a>(
    data: &'a str,
    tokens: &mut Vec<Token<'a>>,
    char_indices: &mut itertools::PeekNth<std::str::CharIndices<'a>>,
    byte_idx: usize,
) {
    // If the following char is also a slash, it's a comment.
    // Consume the rest of the line.
    // Otherwise, it's a simple slash
    if let Some((byte_idx_next, c_next)) = char_indices.peek().cloned()
        && c_next == '/'
    {
        // consume the rest of the line, this is a comment.
        // It's safe to start with consuming, because we know the first character is a slash
        loop {
            char_indices.next();
            match char_indices.peek() {
                Some((_, c)) if *c == '\n' => break, // EOL
                Some(_) => continue,                 // Any part of the comment
                None => break,                       // EOF
            }
        }
    } else {
        tokens.push(Token {
            token_type: TokenType::Slash,
            lexeme: &data[byte_idx..byte_idx + '/'.len_utf8()],
            literal: None,
        })
    }
}

fn handle_greater<'a>(
    data: &'a str,
    tokens: &mut Vec<Token<'a>>,
    char_indices: &mut itertools::PeekNth<std::str::CharIndices<'a>>,
    byte_idx: usize,
) {
    if let Some((byte_idx_next, c_next)) = char_indices.peek().cloned()
        && c_next == '='
    {
        // consume the next char, which is confirmed to be '='
        char_indices.next();
        tokens.push(Token {
            token_type: TokenType::GreaterEqual,
            lexeme: &data[byte_idx..byte_idx_next + '='.len_utf8()],
            literal: None,
        })
    } else {
        tokens.push(Token {
            token_type: TokenType::Greater,
            lexeme: &data[byte_idx..byte_idx + '!'.len_utf8()],
            literal: None,
        })
    }
}

fn handle_less<'a>(
    data: &'a str,
    tokens: &mut Vec<Token<'a>>,
    char_indices: &mut itertools::PeekNth<std::str::CharIndices<'a>>,
    byte_idx: usize,
) {
    if let Some((byte_idx_next, c_next)) = char_indices.peek().cloned()
        && c_next == '='
    {
        // consume the next char, which is confirmed to be '='
        char_indices.next();
        tokens.push(Token {
            token_type: TokenType::LessEqual,
            lexeme: &data[byte_idx..byte_idx_next + '='.len_utf8()],
            literal: None,
        })
    } else {
        tokens.push(Token {
            token_type: TokenType::Less,
            lexeme: &data[byte_idx..byte_idx + '!'.len_utf8()],
            literal: None,
        })
    }
}

fn handle_bang<'a>(
    data: &'a str,
    tokens: &mut Vec<Token<'a>>,
    char_indices: &mut itertools::PeekNth<std::str::CharIndices<'a>>,
    byte_idx: usize,
) {
    if let Some((byte_idx_next, c_next)) = char_indices.peek().cloned()
        && c_next == '='
    {
        // consume the next char, which is confirmed to be '='
        char_indices.next();
        tokens.push(Token {
            token_type: TokenType::BangEqual,
            lexeme: &data[byte_idx..byte_idx_next + '='.len_utf8()],
            literal: None,
        })
    } else {
        tokens.push(Token {
            token_type: TokenType::Bang,
            lexeme: &data[byte_idx..byte_idx + '!'.len_utf8()],
            literal: None,
        })
    }
}

fn handle_equal<'a>(
    data: &'a str,
    tokens: &mut Vec<Token<'a>>,
    char_indices: &mut itertools::PeekNth<std::str::CharIndices<'a>>,
    byte_idx: usize,
) {
    if let Some((byte_idx_next, c_next)) = char_indices.peek().cloned()
        && c_next == '='
    {
        // consume the next char, which is confirmed to be '='
        char_indices.next();
        tokens.push(Token {
            token_type: TokenType::EqualEqual,
            lexeme: &data[byte_idx..byte_idx_next + '='.len_utf8()],
            literal: None,
        })
    } else {
        tokens.push(Token {
            token_type: TokenType::Equal,
            lexeme: &data[byte_idx..byte_idx + '='.len_utf8()],
            literal: None,
        })
    }
}

fn token_from_single_char<'a>(c: &'a str) -> Option<Token<'a>> {
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
