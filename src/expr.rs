use std::fmt::Display;

use itertools::Itertools;

use crate::token::{Token, TokenValue};

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(TokenValue),
    Grouping(Box<Expr>),
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        operator: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Variable(String),
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Expr::Literal(token_value) => token_value.to_string(),
            Expr::Grouping(expr) => format!("(group {expr})"),
            Expr::Unary { operator, right } => format!("({} {right})", operator.lexeme),
            Expr::Binary {
                operator,
                left,
                right,
            } => format!("({} {left} {right})", operator.lexeme),
            Expr::Variable(name) => name.to_string(),
            Expr::Assign { name, value } => format!("({name}={value}"),
            Expr::Logical {
                left,
                operator,
                right,
            } => format!("{left} {operator} {right}"),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => format!(
                "{callee}({})",
                arguments
                    .iter()
                    .map(|s| s.to_string())
                    .collect_vec()
                    .join(",")
            ),
        };
        write!(f, "{output}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{TokenType, TokenValue};

    #[test]
    fn test_display_bool() {
        let value = TokenValue::Boolean(true);
        let expr = Expr::Literal(value);
        assert_eq!(expr.to_string(), "true");
    }

    #[test]
    fn test_display_bang() {
        let operator = Token {
            token_type: TokenType::Bang,
            lexeme: "!".to_string(),
            literal: None,
        };
        let right = Expr::Literal(TokenValue::Boolean(true));
        let expr = Expr::Unary {
            operator,
            right: Box::new(right),
        };
        assert_eq!(expr.to_string(), "(! true)");
    }
}
