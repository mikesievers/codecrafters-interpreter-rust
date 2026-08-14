pub struct Token<'a> {
    pub token_type: TokenType,
    pub lexeme: &'a str,
    // The literal will in the future hold a TokenValue, for now
    // it holds only 'null'
    pub literal: Option<()>,
}

impl Token<'_> {
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

pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    // LeftBrace,
    // RightBrace,
    // Comma,
    // Dot,
    // Minus,
    // Plus,
    // Semicolon,
    // Slash,
    // Star,

    // // One or two character tokens.
    // Bang,
    // BangEqual,
    // Equal,
    // EqualEqual,
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
            TokenType::RightParen => "RIGHT_PARENT",
            TokenType::Eof => "EOF",
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
