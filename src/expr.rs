use std::fmt::Display;

use itertools::Itertools;

use crate::token::{Token, TokenValue};

pub struct ExprId(usize);

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Literal(TokenValue),
    Grouping(Box<ExprKind>),
    Logical {
        left: Box<ExprKind>,
        operator: Token,
        right: Box<ExprKind>,
    },
    Unary {
        operator: Token,
        right: Box<ExprKind>,
    },
    Binary {
        operator: Token,
        left: Box<ExprKind>,
        right: Box<ExprKind>,
    },
    Variable(String),
    Assign {
        name: String,
        value: Box<ExprKind>,
    },
    Call {
        callee: Box<ExprKind>,
        paren: Token,
        arguments: Vec<ExprKind>,
    },
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            ExprKind::Literal(token_value) => token_value.to_string(),
            ExprKind::Grouping(expr) => format!("(group {expr})"),
            ExprKind::Unary { operator, right } => format!("({} {right})", operator.lexeme),
            ExprKind::Binary {
                operator,
                left,
                right,
            } => format!("({} {left} {right})", operator.lexeme),
            ExprKind::Variable(name) => name.to_string(),
            ExprKind::Assign { name, value } => format!("({name}={value}"),
            ExprKind::Logical {
                left,
                operator,
                right,
            } => format!("{left} {operator} {right}"),
            ExprKind::Call {
                callee,
                paren,
                arguments,
            } => format!(
                "{callee}({})",
                arguments
                    .iter()
                    .map(std::string::ToString::to_string)
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
        let expr = ExprKind::Literal(value);
        assert_eq!(expr.to_string(), "true");
    }

    #[test]
    fn test_display_bang() {
        let operator = Token {
            token_type: TokenType::Bang,
            lexeme: "!".to_string(),
            literal: None,
        };
        let right = ExprKind::Literal(TokenValue::Boolean(true));
        let expr = ExprKind::Unary {
            operator,
            right: Box::new(right),
        };
        assert_eq!(expr.to_string(), "(! true)");
    }
}
