use std::fmt::Display;

use crate::{evaluate::Evaluate, expr::Expr, lox_error::LoxError};

pub enum Stmt<'a> {
    Expression(Expr<'a>),
    Print(Expr<'a>),
    Var {
        name: &'a str,
        initializer: Option<Expr<'a>>,
    },
}

impl Display for Stmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Expression(expr) => write!(f, "{expr}"),
            Stmt::Print(expr) => write!(f, "print {expr}"),
            Stmt::Var { name, initializer } => match initializer {
                Some(expr) => write!(f, "var {name} = {expr}"),
                None => write!(f, "var {name}"),
            },
        }
    }
}

// TODO: Move to Interpreter and then delete here
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
