use std::collections::HashMap;

use crate::{
    expr::{Expr, ExprId, ExprKind},
    lox_error::LoxError,
    stmt::Stmt,
};

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    locals: HashMap<ExprId, usize>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            scopes: Vec::new(),
            locals: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, program: &Vec<Stmt>) -> Result<HashMap<ExprId, usize>, LoxError> {
        for stmt in program {
            self.resolve_stmt(stmt)?;
        }

        Ok(self.locals.clone())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), LoxError> {
        match stmt {
            Stmt::Block(stmts) => {
                self.begin_scope();
                for stmt in stmts {
                    self.resolve_stmt(stmt)?;
                }
                self.end_scope();
            }
            Stmt::Expression(expr) => todo!(),
            Stmt::Function { name, params, body } => {
                self.declare(&name.lexeme);
                self.define(&name.lexeme);

                self.begin_scope();
                for param in params {
                    self.declare(&param.lexeme);
                    self.define(&param.lexeme);
                }
                self.resolve_stmt(body)?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => todo!(),
            Stmt::Print(expr) => todo!(),
            Stmt::Return { keyword, value } => todo!(),
            Stmt::Var { name, initializer } => {
                self.declare(name);
                if let Some(initializer) = initializer {
                    self.resolve_expr(initializer)?;
                }
                self.define(name);
            }
            Stmt::While { condition, body } => todo!(),
        }
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), LoxError> {
        match &expr.kind {
            ExprKind::Literal(token_value) => todo!(),
            ExprKind::Grouping(expr) => todo!(),
            ExprKind::Logical {
                left,
                operator,
                right,
            } => todo!(),
            ExprKind::Unary { operator, right } => todo!(),
            ExprKind::Binary {
                operator,
                left,
                right,
            } => todo!(),
            ExprKind::Variable(name) => {
                if let Some(scope) = self.scopes.last()
                    && let Some(is_defined) = scope.get(name)
                    && !is_defined
                {
                    return Err(LoxError::SyntaxError(
                        "Can't read local variable in its own initializer.".to_string(),
                    ));
                }
                self.resolve_local(expr, name);
            }
            ExprKind::Assign { name, value } => {
                self.resolve_expr(value)?;
                self.resolve_local(expr, name);
            }
            ExprKind::Call {
                callee,
                paren,
                arguments,
            } => todo!(),
        }
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str) {
        // A variable has been declared but not defined -> key exists, but value is false
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), false);
        }
    }

    fn define(&mut self, name: &str) {
        // A variable has been defined -> Set the entry to true
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), true);
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &str) {
        for (distance, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.locals.insert(expr.id, distance);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    pub fn test_resolver() {
        let mut resolver = Resolver::new();
    }
}
