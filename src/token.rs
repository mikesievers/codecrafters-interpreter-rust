use std::fmt::Display;

pub struct Token<'a> {
    pub token_type: TokenType,
    pub lexeme: &'a str,
    // The literal will in the future hold a TokenValue, for now
    // it holds only 'null'
    pub literal: Option<()>,
}

impl<'a> Token<'a> {
    pub fn display(&self) -> String {
        let literal = match self.literal {
            Some(_) => "TBD".to_string(),
            None => "null".to_string(),
        };
        format!(
            "{} {} {}",
            self.token_type.display_name(),
            self.lexeme,
            literal
        )
    }

    pub fn eof() -> Self {
        Token {
            token_type: TokenType::Eof,
            lexeme: "",
            literal: None,
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Star,
    // Slash,

    // // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    // Greater,
    // GreaterEqual,
    // Less,
    // LessEqual,

    // // Literals.
    // Identifier,
    // String,
    // Number,

    // // Keywords.
    // And,
    // Class,
    // Else,
    // False,
    // Fun,
    // For,
    // If,
    // Nil,
    // Or,
    // Print,
    // Return,
    // Super,
    // This,
    // True,
    // Var,
    // While,
    Eof,
}

impl TokenType {
    fn display_name(&self) -> &str {
        match self {
            TokenType::LeftParen => "LEFT_PAREN",
            TokenType::RightParen => "RIGHT_PAREN",
            TokenType::Eof => "EOF",
            TokenType::LeftBrace => "LEFT_BRACE",
            TokenType::RightBrace => "RIGHT_BRACE",
            TokenType::Comma => "COMMA",
            TokenType::Dot => "DOT",
            TokenType::Minus => "MINUS",
            TokenType::Plus => "PLUS",
            TokenType::Semicolon => "SEMICOLON",
            TokenType::Star => "STAR",
            TokenType::Equal => "EQUAL",
            TokenType::EqualEqual => "EQUAL_EQUAL",
            TokenType::Bang => "BANG",
            TokenType::BangEqual => "BANG_EQUAL",
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_display() {
        let token = Token {
            token_type: TokenType::LeftParen,
            lexeme: "(",
            literal: None,
        };

        assert_eq!(token.display(), "LEFT_PAREN ( null");
    }
}
