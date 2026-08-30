use std::fs::read_to_string;

use anyhow::Result;
use itertools::peek_nth;

use crate::token::{Token, TokenType, TokenValue};

pub struct Scanner {
    data: String,
}

impl Scanner {
    #[must_use]
    pub fn from_string(data: String) -> Self {
        Scanner { data }
    }

    /// Creates a scanner from the contents of a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn from_file(filename: &String) -> Result<Self> {
        let data = read_to_string(filename)?;
        Ok(Scanner { data })
    }

    // The lexemes are references into self.data, therefore
    // the returned tokens borrow from self while they are in use.
    //
    // Returns the tokens together with a flag indicating whether any
    // lexical errors were encountered.
    #[must_use]
    pub fn tokenize(&self) -> (Vec<Token<'_>>, bool) {
        let mut line_no: u32 = 1;
        let mut tokens: Vec<Token<'_>> = vec![];
        let mut had_errors = false;

        let mut char_indices = peek_nth(self.data.char_indices());

        loop {
            match char_indices.next() {
                Some((byte_idx, c))
                    if let Some(token) =
                        token_from_single_char(&self.data[byte_idx..byte_idx + c.len_utf8()]) =>
                {
                    tokens.push(token);
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
                Some((_byte_idx, c)) if c == ' ' || c == '\t' || c == '\n' => {
                    if c == '\n' {
                        line_no += 1;
                    }
                }
                // '"' String
                Some((byte_idx, '"')) => {
                    if let Some(n) =
                        handle_string(&self.data, &mut tokens, &mut char_indices, byte_idx)
                    {
                        line_no += n;
                    } else {
                        eprintln!("[line {line_no}] Error: Unterminated string.");
                        had_errors = true;
                    }
                }
                // Number
                Some((byte_idx, c)) if c.is_ascii_digit() => {
                    handle_number(&self.data, &mut tokens, &mut char_indices, byte_idx);
                }
                // Identifier
                Some((byte_idx, c)) if is_alpha(c) => {
                    handle_identifier(&self.data, &mut tokens, &mut char_indices, byte_idx, c);
                }
                // -- Everthing else
                // default: emit error message
                Some((_byte_idx, c)) => {
                    eprintln!("[line {line_no}] Error: Unexpected character: {c}");
                    had_errors = true;
                }
                // No more chars -> EOF and break
                None => {
                    tokens.push(Token::eof());
                    break;
                }
            }
        }

        (tokens, had_errors)
    }
}

fn is_alpha(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == '_'
}

fn is_alpha_numeric(c: char) -> bool {
    is_alpha(c) || c.is_ascii_digit()
}

fn handle_identifier<'a>(
    data: &'a str,
    tokens: &mut Vec<Token<'a>>,
    char_indices: &mut itertools::PeekNth<std::str::CharIndices<'a>>,
    byte_idx: usize,
    c: char,
) {
    // Identifier: anything alphanumeric, starting with alpha
    let mut byte_len = c.len_utf8();

    loop {
        match char_indices.peek() {
            Some((byte_idx_next, c_next)) if is_alpha_numeric(*c_next) => {
                byte_len += c_next.len_utf8();
                char_indices.next();
            }
            _ => {
                break;
            }
        }
    }

    let lexeme = &data[byte_idx..byte_idx + byte_len];
    let token_type = match TokenType::from_str(lexeme) {
        Some(token_type) => token_type,
        None => TokenType::Identifier,
    };

    tokens.push(Token {
        token_type,
        lexeme,
        literal: None,
    });
}

fn handle_number<'a>(
    data: &'a str,
    tokens: &mut Vec<Token<'a>>,
    char_indices: &mut itertools::PeekNth<std::str::CharIndices<'a>>,
    byte_idx: usize,
) {
    // Number: digits or digits DOT digits
    // Track the byte length, initialize with a single digit length
    // (0 is taken as an example, all digits assumed to occupy same amount of bytes)
    let mut byte_len = '0'.len_utf8();
    loop {
        match char_indices.peek().copied() {
            Some((_idx_next, c_next)) if c_next.is_ascii_digit() => {
                // Next char is a digit - consume it and increment byte_len
                byte_len += c_next.len_utf8();
                char_indices.next();
            }
            // If the next character is a DOT, and the one after it a digit,
            // consume both
            Some((_idx_next, '.'))
                if let Some((_idx_next_next, c_next_next)) = char_indices.peek_nth(1)
                    && c_next_next.is_ascii_digit() =>
            {
                byte_len += '.'.len_utf8();
                char_indices.next();
                // again use 0 as representative for the length of any other
                // digit char
                byte_len += '0'.len_utf8();
                char_indices.next();
            }
            None | Some(_) => {
                break;
            }
        }
    }

    let number_chars = &data[byte_idx..byte_idx + byte_len];

    tokens.push(Token {
        token_type: TokenType::Number,
        lexeme: number_chars,
        // SAFETY: The above already checks whether the number consists only of digits or digits DOT
        // digits, therefor unwrap is safe
        literal: Some(TokenValue::Number(number_chars.parse::<f64>().unwrap())),
    });
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
        if let Some((_byte_idx_next, c_next)) = char_indices.next() {
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
    if let Some((_byte_idx_next, c_next)) = char_indices.peek().copied()
        && c_next == '/'
    {
        // consume the rest of the line, this is a comment.
        // It's safe to start with consuming, because we know the first character is a slash
        loop {
            char_indices.next();
            match char_indices.peek() {
                Some((_, c)) if *c == '\n' => break, // EOL
                Some(_) => {}                        // Any part of the comment
                None => break,                       // EOF
            }
        }
    } else {
        tokens.push(Token {
            token_type: TokenType::Slash,
            lexeme: &data[byte_idx..byte_idx + '/'.len_utf8()],
            literal: None,
        });
    }
}

fn handle_greater<'a>(
    data: &'a str,
    tokens: &mut Vec<Token<'a>>,
    char_indices: &mut itertools::PeekNth<std::str::CharIndices<'a>>,
    byte_idx: usize,
) {
    if let Some((byte_idx_next, c_next)) = char_indices.peek().copied()
        && c_next == '='
    {
        // consume the next char, which is confirmed to be '='
        char_indices.next();
        tokens.push(Token {
            token_type: TokenType::GreaterEqual,
            lexeme: &data[byte_idx..byte_idx_next + '='.len_utf8()],
            literal: None,
        });
    } else {
        tokens.push(Token {
            token_type: TokenType::Greater,
            lexeme: &data[byte_idx..byte_idx + '>'.len_utf8()],
            literal: None,
        });
    }
}

fn handle_less<'a>(
    data: &'a str,
    tokens: &mut Vec<Token<'a>>,
    char_indices: &mut itertools::PeekNth<std::str::CharIndices<'a>>,
    byte_idx: usize,
) {
    if let Some((byte_idx_next, c_next)) = char_indices.peek().copied()
        && c_next == '='
    {
        // consume the next char, which is confirmed to be '='
        char_indices.next();
        tokens.push(Token {
            token_type: TokenType::LessEqual,
            lexeme: &data[byte_idx..byte_idx_next + '='.len_utf8()],
            literal: None,
        });
    } else {
        tokens.push(Token {
            token_type: TokenType::Less,
            lexeme: &data[byte_idx..byte_idx + '<'.len_utf8()],
            literal: None,
        });
    }
}

fn handle_bang<'a>(
    data: &'a str,
    tokens: &mut Vec<Token<'a>>,
    char_indices: &mut itertools::PeekNth<std::str::CharIndices<'a>>,
    byte_idx: usize,
) {
    if let Some((byte_idx_next, c_next)) = char_indices.peek().copied()
        && c_next == '='
    {
        // consume the next char, which is confirmed to be '='
        char_indices.next();
        tokens.push(Token {
            token_type: TokenType::BangEqual,
            lexeme: &data[byte_idx..byte_idx_next + '='.len_utf8()],
            literal: None,
        });
    } else {
        tokens.push(Token {
            token_type: TokenType::Bang,
            lexeme: &data[byte_idx..byte_idx + '!'.len_utf8()],
            literal: None,
        });
    }
}

fn handle_equal<'a>(
    data: &'a str,
    tokens: &mut Vec<Token<'a>>,
    char_indices: &mut itertools::PeekNth<std::str::CharIndices<'a>>,
    byte_idx: usize,
) {
    if let Some((byte_idx_next, c_next)) = char_indices.peek().copied()
        && c_next == '='
    {
        // consume the next char, which is confirmed to be '='
        char_indices.next();
        tokens.push(Token {
            token_type: TokenType::EqualEqual,
            lexeme: &data[byte_idx..byte_idx_next + '='.len_utf8()],
            literal: None,
        });
    } else {
        tokens.push(Token {
            token_type: TokenType::Equal,
            lexeme: &data[byte_idx..byte_idx + '='.len_utf8()],
            literal: None,
        });
    }
}

fn token_from_single_char(c: &str) -> Option<Token<'_>> {
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
        let scanner = Scanner::from_string("()".into());

        let (tokens, had_errors) = scanner.tokenize();
        assert!(!had_errors);
        assert_eq!(
            tokens.iter().map(|token| { token.display() }).collect_vec(),
            vec!["LEFT_PAREN ( null", "RIGHT_PAREN ) null", "EOF  null"]
        );
    }
}
