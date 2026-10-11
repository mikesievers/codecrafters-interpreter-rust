use std::fmt::Display;

use itertools::Itertools;

use crate::token::{Token, TokenValue};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExprId(pub usize);

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub id: ExprId,
    pub kind: ExprKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
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
        write!(f, "{}", self.kind)
    }
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
        let right = Expr {
            id: ExprId(0),
            kind: ExprKind::Literal(TokenValue::Boolean(true)),
        };
        let expr = Expr {
            id: ExprId(1),
            kind: ExprKind::Unary {
                operator,
                right: Box::new(right),
            },
        };
        assert_eq!(expr.to_string(), "(! true)");
    }
}
