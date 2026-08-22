use crate::{
    expr::Expr,
    token::{Token, TokenType, TokenValue},
};

static EOF_TOKEN: Token<'static> = Token {
    token_type: TokenType::Eof,
    lexeme: "",
    literal: None,
};

#[derive(Debug)]
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
        parse_expression(self)
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
            _ => self.tokens.get(self.current - 1).unwrap(),
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

    fn consume(&mut self, token_type: &TokenType) -> Result<(), ()> {
        if self.peek().token_type == *token_type {
            self.advance();
            Ok(())
        } else {
            eprintln!("[line 1] Missing closing parenthesis");
            Err(())
        }
    }
}

// Implementation of the different steps of the grammar

// expression     → equality ;
fn parse_expression<'a>(parser: &mut Parser<'a>) -> Result<Expr<'a>, ()> {
    parse_equality(parser)
}

// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
fn parse_equality<'a>(parser: &mut Parser<'a>) -> Result<Expr<'a>, ()> {
    parse_comparison(parser)
}

// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
fn parse_comparison<'a>(parser: &mut Parser<'a>) -> Result<Expr<'a>, ()> {
    parse_term(parser)
}

// term           → factor ( ( "-" | "+" ) factor )* ;
fn parse_term<'a>(parser: &mut Parser<'a>) -> Result<Expr<'a>, ()> {
    let expr = parse_factor(parser)?;

    if parser.matches(&[TokenType::Minus, TokenType::Plus]) {
        let operator = parser.previous().clone();
        let right = parse_factor(parser)?;
        return Ok(Expr::Binary{operator, left: Box::new(expr), right: Box::new(right)})
    }

    Ok(expr)
}

// factor         → unary ( ( "/" | "*" ) unary )* ;
fn parse_factor<'a>(parser: &mut Parser<'a>) -> Result<Expr<'a>, ()> {
    let expr = parse_unary(parser)?;

    if parser.matches(&[TokenType::Slash, TokenType::Star]) {
        let operator = parser.previous().clone();
        let right = parse_unary(parser)?;
        return Ok(Expr::Binary{ operator, left: Box::new(expr), right: Box::new(right)});
    }

    Ok(expr)
}

// unary          → ( "!" | "-" ) unary
//                | primary ;
fn parse_unary<'a>(parser: &mut Parser<'a>) -> Result<Expr<'a>, ()> {
    if parser.matches(&[TokenType::Bang, TokenType::Minus]) {
        let operator = parser.previous().clone();
        let right = parse_unary(parser)?;
        return Ok(Expr::Unary{ operator, right: Box::new(right) })
    }

    parse_primary(parser)
}

// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;
fn parse_primary<'a>(parser: &mut Parser<'a>) -> Result<Expr<'a>, ()> {
    eprintln!("{}",parser.peek());
    if parser.matches(&[TokenType::Number]) {
        let n = parser
            .previous()
            .literal
            .clone()
            .expect("Number token without literal must not exist");
        return Ok(Expr::Literal(n));
    }

    if parser.matches(&[TokenType::String]) {
        let s = parser
            .previous()
            .literal
            .clone()
            .expect("String token without literal must not exist");
        return Ok(Expr::Literal(s));
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

    if parser.matches(&[TokenType::LeftParen]) {
        let expr = parse_expression(parser)?;
        parser.consume(&TokenType::RightParen)?;
        return Ok(Expr::Grouping(Box::new(expr)));
    }

    eprintln!("No matching primary found");
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Token, TokenType};

    #[test]
    fn test_parse_primary_nil() {
        let token = Token {
            token_type: TokenType::Nil,
            lexeme: "nil",
            literal: None,
        };
        let tokens = vec![token, Token::eof()];

        let mut parser = Parser::new(tokens);

        let primary = parse_primary(&mut parser).unwrap();
        assert_eq!(primary.to_string(), "nil".to_string());
    }

    #[test]
    fn test_parse_primary_number() {
        let token = Token {
            token_type: TokenType::Number,
            lexeme: "",
            literal: Some(TokenValue::Number(42.0)),
        };
        let tokens = vec![token, Token::eof()];

        let mut parser = Parser::new(tokens);

        let primary = parse_primary(&mut parser).unwrap();
        assert_eq!(primary.to_string(), "42.0".to_string());
    }

    #[test]
    fn test_parse_primary_string() {
        let token = Token {
            token_type: TokenType::String,
            lexeme: "",
            literal: Some(TokenValue::String("42")),
        };
        let tokens = vec![token, Token::eof()];

        let mut parser = Parser::new(tokens);

        let primary = parse_primary(&mut parser).unwrap();
        assert_eq!(primary.to_string(), "42".to_string());
    }

    #[test]
    fn test_parse_primary_grouping() {
        let left_paren = Token {
            token_type: TokenType::LeftParen,
            lexeme: "(",
            literal: None,
        };
        let token = Token {
            token_type: TokenType::String,
            lexeme: "",
            literal: Some(TokenValue::String("foo")),
        };
        let right_paren = Token {
            token_type: TokenType::RightParen,
            lexeme: ")",
            literal: None,
        };
        let tokens = vec![left_paren, token, right_paren, Token::eof()];

        let mut parser = Parser::new(tokens);

        let primary = parser.parse().unwrap();
        assert_eq!(primary.to_string(), "(group foo)".to_string());
    }

    #[test]
    fn test_parse_unary() {
        let bang = Token {
            token_type: TokenType::Bang,
            lexeme: "!",
            literal: None,
        };
        let token_true = Token {
            token_type: TokenType::True,
            lexeme: "",
            literal: None,
        };
        let tokens = vec![bang.clone(), bang, token_true, Token::eof()];

        let mut parser = Parser::new(tokens);

        let primary = parser.parse().unwrap();
        assert_eq!(primary.to_string(), "(! (! true))".to_string());
    }

    #[test]
    fn test_parse_binary() {
        let token_left = Token {
            token_type: TokenType::Number,
            lexeme: "40",
            literal: Some(TokenValue::Number(40.0)),
        };
        let plus = Token {
            token_type: TokenType::Plus,
            lexeme: "+",
            literal: None,
        };
        let token_right = Token {
            token_type: TokenType::Number,
            lexeme: "2",
            literal: Some(TokenValue::Number(2.0)),
        };
        let tokens = vec![token_left, plus, token_right, Token::eof()];

        let mut parser = Parser::new(tokens);

        let primary = parser.parse().unwrap();
        assert_eq!(primary.to_string(), "(+ 40.0 2.0)".to_string());
    }
}
