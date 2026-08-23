use std::fmt::Display;

use crate::lox_value::LoxValue;
use crate::token::{Token, TokenType, TokenValue};
use crate::evaluate::Evaluate;

pub enum Expr<'a> {
    Literal(TokenValue<'a>),
    Grouping(Box<Expr<'a>>),
    Unary { operator: Token<'a>, right: Box<Expr<'a>> },
    Binary { operator: Token<'a>, left: Box<Expr<'a>>, right: Box<Expr<'a>> },
}

impl Display for Expr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Expr::Literal(token_value) => token_value.to_string(),
            Expr::Grouping(expr) => format!("(group {expr})"),
            Expr::Unary { operator, right } => format!("({} {right})", operator.lexeme),
            Expr::Binary { operator, left, right } => format!("({} {left} {right})", operator.lexeme),
        };
        write!(f, "{output}")
    }
}

impl Evaluate for Expr<'_> {
    fn evaluate(&self) -> crate::lox_value::LoxValue {
        match self {
            Expr::Literal(token_value) => {
                match token_value {
                    TokenValue::String(s) => LoxValue::String(s.to_string()),
                    TokenValue::Number(n) => LoxValue::Number(*n),
                    TokenValue::Boolean(b) => LoxValue::Boolean(*b),
                    TokenValue::Nil => LoxValue::Nil,
                }
            },
            Expr::Grouping(grp) => grp.evaluate(),
            Expr::Unary { operator, right } => {
                match operator.token_type {
                    TokenType::Minus => {-right.evaluate()},
                    TokenType::Bang => { !right.evaluate()},
                    _ => {panic!("Unexpected Unary operator encountered")},
                }
            } ,
            Expr::Binary { operator, left, right } => {
                match operator.token_type {
                    TokenType::Minus => left.evaluate() - right.evaluate(),
                    TokenType::Plus => todo!(),
                    TokenType::Star => left.evaluate() * right.evaluate(),
                    TokenType::Slash => todo!(),
                    TokenType::Equal => todo!(),
                    TokenType::EqualEqual => todo!(),
                    TokenType::Greater => todo!(),
                    TokenType::GreaterEqual => todo!(),
                    TokenType::Less => todo!(),
                    TokenType::LessEqual => todo!(),
                    _ => {panic!("Unexpected Binary operator encountered")},
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lox_value::LoxValue, token::{TokenType, TokenValue}};

    #[test]
    fn test_display_bool() {
        let value = TokenValue::Boolean(true);
        let expr = Expr::Literal(value);
        assert_eq!(expr.to_string(), "true");
    }

    #[test]
    fn test_display_bang() {
        let operator = Token { token_type: TokenType::Bang, lexeme: "!", literal: None };
        let right = Expr::Literal(TokenValue::Boolean(true));
        let expr = Expr::Unary{operator, right: Box::new(right)};
        assert_eq!(expr.to_string(), "(! true)");
    }

    // Evaluation
    #[test]
    fn test_eval_literal() {
        let expr = Expr::Literal(TokenValue::Boolean(true));
        assert_eq!(expr.evaluate(), LoxValue::Boolean(true));

    }

    #[test]
    fn test_minus_number() {
        let left = Box::new(Expr::Literal(TokenValue::Number(44.0)));
        let right = Box::new(Expr::Literal(TokenValue::Number(2.0)));
        let sub = Expr::Binary { operator: Token{ token_type: TokenType::Minus, lexeme: "-", literal: None}, left, right };
        assert_eq!(sub.evaluate(), LoxValue::Number(42.0));
    }

    #[test]
    fn test_slash_number() {
        let left = Box::new(Expr::Literal(TokenValue::Number(4.0)));
        let right = Box::new(Expr::Literal(TokenValue::Number(2.0)));
        let sub = Expr::Binary { operator: Token{ token_type: TokenType::Star, lexeme: "*", literal: None}, left, right };
        assert_eq!(sub.evaluate(), LoxValue::Number(8.0));
    }
}
