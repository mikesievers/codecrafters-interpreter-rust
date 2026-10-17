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
        // The global environment is also a global scope
        self.begin_scope();
        for stmt in program {
            self.resolve_stmt(stmt)?;
        }
        self.end_scope();

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
            Stmt::Expression(expr) => self.resolve_expr(expr)?,
            Stmt::Function { name, params, body } => {
                self.declare(&name.lexeme);
                self.define(&name.lexeme);

                self.begin_scope();
                for param in params {
                    self.declare(&param.lexeme);
                    self.define(&param.lexeme);
                }
                // resolve the body
                // Ensure that no new scope is started
                // self.resolve_stmt(body)?;
                match &**body {
                    Stmt::Block(stmts) => {
                        for stmt in stmts {
                            self.resolve_stmt(stmt)?;
                        }
                    }
                    _ => self.resolve_stmt(body)?,
                }
                self.end_scope();
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(else_branch)?;
                }
            }
            Stmt::Print(expr) => {
                self.resolve_expr(expr)?;
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    self.resolve_expr(value)?;
                }
            }
            Stmt::Var { name, initializer } => {
                self.declare(name);
                if let Some(initializer) = initializer {
                    self.resolve_expr(initializer)?;
                }
                self.define(name);
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)?;
            }
        }
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), LoxError> {
        match &expr.kind {
            ExprKind::Literal(_) => {}
            ExprKind::Grouping(expr) => {
                self.resolve_expr(expr)?;
            }
            ExprKind::Logical { left, right, .. } | ExprKind::Binary { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            ExprKind::Unary { right, .. } => {
                self.resolve_expr(right)?;
            }
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
                callee, arguments, ..
            } => {
                self.resolve_expr(callee)?;
                for arg in arguments {
                    self.resolve_expr(arg)?;
                }
            }
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
                return;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Scanner, parser::Parser};

    use super::*;

    #[test]
    pub fn test_resolver() {
        let scanner = Scanner::from_file(&"testdata/test_resolve_1.lox".to_string()).unwrap();

        let (tokens, _) = scanner.tokenize();

        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        let mut resolver = Resolver::new();
        // TODO: use the resulting locals map
        let locals = resolver.resolve(&program).unwrap();

        assert_eq!("todo","Test the resolver result");
    }
}
