use crate::{
    expr::Expr,
    token::{Token, TokenType, TokenValue},
};

static EOF_TOKEN: Token<'static> = Token {
    token_type: TokenType::Eof,
    lexeme: "",
    literal: None,
};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        // Make sure the tokens Vec ends with an Eof
        assert!(
            tokens.last() == Some(&EOF_TOKEN),
            "The tokens vector must end with an Eof token"
        );

        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Expr<'_>, ()> {
        parse_primary(self)
    }

    fn advance(&mut self) -> &Token<'a> {
        if !(self.is_at_end()) {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.current == self.tokens.len() - 1
    }

    fn peek(&'_ self) -> &Token<'a> {
        // SAFETY: self.current is only ever incremented by self.advance,
        // and only if it is not already pointing to the last token
        // Also, the Parser panic()s if the tokens Vec does not contain one
        // (EOF) token
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> &Token<'a> {
        match self.current {
            0 => &EOF_TOKEN,
            // SAFETY: self.current is > 0 here and only advanced through
            // advance, which ensures it is does not go out of bounds
            _ => self.tokens.get(self.current).unwrap(),
        }
    }

    fn matches(&mut self, token_types: &[TokenType]) -> bool {
        // SAFETY: self.current can only be 0 to tokens.len() and
        // tokens has at least one element via new()
        if token_types.contains(&self.tokens.get(self.current).unwrap().token_type) {
            self.advance();
            return true;
        }
        false
    }
}

// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;
fn parse_primary<'a>(parser: &mut Parser<'a>) -> Result<Expr<'a>, ()> {
    if parser.matches(&[TokenType::Number]) {
        let token = parser.peek();
        let token_value = token
            .literal
            .clone()
            .expect("Number token without literal must not exist");
        return Ok(Expr::Literal(token_value));
    }

    if parser.matches(&[TokenType::Nil]) {
        return Ok(Expr::Literal(TokenValue::Nil));
    }

    if parser.matches(&[TokenType::True]) {
        return Ok(Expr::Literal(TokenValue::Boolean(true)));
    }

    if parser.matches(&[TokenType::False]) {
        return Ok(Expr::Literal(TokenValue::Boolean(false)));
    }
    eprintln!("No matching primary found");
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Token, TokenType};

    #[test]
    fn test_parse_primary() {
        let token = Token {
            token_type: TokenType::Nil,
            lexeme: "nil",
            literal: None,
        };
        let tokens = vec![token, Token::eof()];

        let mut parser = Parser::new(tokens);

        let ast = parser.parse().unwrap();
        assert_eq!(ast.to_string(), "nil".to_string());
    }
}
