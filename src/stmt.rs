use std::fmt::{Display, Error};

use crate::{
    evaluate::Evaluate,
    expr::Expr,
    lox_error::LoxError,
    token::{Token, TokenValue},
};

pub enum Stmt<'a> {
    Expression(Expr<'a>),
    Print(Expr<'a>),
    Var {
        name: Token<'a>,
        initializer: Option<Expr<'a>>,
    },
}

impl Display for Stmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Expression(expr) => write!(f, "{expr}"),
            Stmt::Print(expr) => write!(f, "print {expr}"),
            Stmt::Var { name, initializer } => {
                let Some(TokenValue::String(name_value)) = name.literal else {
                    return Err(Error);
                };

                match initializer {
                    Some(expr) => write!(f, "var {name_value} = {expr}"),
                    None => write!(f, "var {name_value}"),
                }
            }
        }
    }
}

impl Stmt<'_> {
    pub fn execute(&self) -> Result<(), LoxError> {
        match self {
            Stmt::Expression(expr) => match expr.evaluate() {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            Stmt::Print(expr) => {
                println!("{}", expr.evaluate()?);
                Ok(())
            }
            Stmt::Var { name, initializer } => todo!(),
        }
    }
}
