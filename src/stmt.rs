use std::fmt::Display;

use crate::expr::Expr;

pub enum Stmt<'a> {
    Block(Vec<Stmt<'a>>),
    Expression(Expr<'a>),
    If {
        condition: Expr<'a>,
        then_branch: Box<Stmt<'a>>,
        else_branch: Option<Box<Stmt<'a>>>,
    },
    Print(Expr<'a>),
    Var {
        name: &'a str,
        initializer: Option<Expr<'a>>,
    },
    While {
        condition: Expr<'a>,
        body: Box<Stmt<'a>>,
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
                let inner_block: String =
                    stmts.iter().map(std::string::ToString::to_string).collect();
                write!(f, "{{\n{inner_block}\n}}")
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let else_string: String;
                if let Some(else_stmt) = else_branch {
                    else_string = format!("\nelse\n   {else_stmt}");
                } else {
                    else_string = String::new();
                }

                write!(f, "if ({condition}) \n   {then_branch}{else_string}")
            }
            Stmt::While { condition, body } => write!(f, "while ({condition}) {body}"),
        }
    }
}
