use std::fmt::Display;

use itertools::Itertools;

use crate::token::{Token, TokenValue};

#[derive(Clone, Debug, PartialEq)]
pub enum Expr<'a> {
    Literal(TokenValue<'a>),
    Grouping(Box<Expr<'a>>),
    Logical {
        left: Box<Expr<'a>>,
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Unary {
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Binary {
        operator: Token<'a>,
        left: Box<Expr<'a>>,
        right: Box<Expr<'a>>,
    },
    Variable(&'a str),
    Assign {
        name: &'a str,
        value: Box<Expr<'a>>,
    },
    Call {
        callee: Box<Expr<'a>>,
        paren: Token<'a>,
        arguments: Vec<Expr<'a>>,
    },
}

impl Display for Expr<'_> {
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
            lexeme: "!",
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
