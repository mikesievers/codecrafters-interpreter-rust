use std::fmt::Display;

use crate::expr::Expr;

pub enum Stmt<'a> {
    Block(Vec<Stmt<'a>>),
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
            Stmt::Block(stmts) => {
                let inner_block: String = stmts.iter().map(|s| s.to_string()).collect();
                write!(f, "{{\n{inner_block}\n}}")
            }
        }
    }
}
