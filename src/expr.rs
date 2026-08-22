use std::fmt::Display;

use crate::token::TokenValue;

pub enum Expr<'a> {
    Literal(TokenValue<'a>),
}

impl Display for Expr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Expr::Literal(token_value) => token_value.to_string(),
        };
        write!(f, "{output}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenValue;

    #[test]
    fn test_display() {
        let value = TokenValue::Boolean(true);
        let expr = Expr::Literal(value);
        assert_eq!(expr.to_string(), "true");
    }
}
