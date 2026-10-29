use std::collections::HashMap;

use crate::{
    expr::{Expr, ExprId, ExprKind},
    lox_error::LoxError,
    stmt::Stmt,
};

enum FunctionType {
    Function,
}

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    locals: HashMap<ExprId, usize>,
    current_function: Option<FunctionType>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            scopes: Vec::new(),
            locals: HashMap::new(),
            current_function: None,
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
            Stmt::Expression(expr) => self.resolve_expr(expr)?,
            Stmt::Function { name, params, body } => {
                self.declare(&name.lexeme)?;
                self.define(&name.lexeme);

                let enclosing_function = self.current_function.replace(FunctionType::Function);

                self.begin_scope();
                for param in params {
                    self.declare(&param.lexeme)?;
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

                self.current_function = enclosing_function;
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
                if self.current_function.is_none() {
                    return Err(LoxError::SyntaxError(
                        "Can't return from top-level code.".into(),
                    ));
                }
                if let Some(value) = value {
                    self.resolve_expr(value)?;
                }
            }
            Stmt::Var { name, initializer } => {
                self.declare(name)?;
                if let Some(initializer) = initializer {
                    self.resolve_expr(initializer)?;
                }
                self.define(name);
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)?;
            }
            Stmt::Class { id, name, methods } => {
                self.declare(&name.lexeme)?;
                self.define(&name.lexeme);
                // Because the class is being declared in this statement, the distance must be 0
                if !self.scopes.is_empty() {
                    self.locals.insert(*id, 0);
                }
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
            ExprKind::Get { object, .. } => {
                self.resolve_expr(object)?;
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

    fn declare(&mut self, name: &str) -> Result<(), LoxError> {
        // if the variable already exists, it's an error
        if let Some(scope) = self.scopes.last()
            && scope.contains_key(name)
        {
            return Err(LoxError::SyntaxError(
                "Already a variable with this name in scope".into(),
            ));
        }
        // A variable has been declared but not defined -> key exists, but value is false
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), false);
        }
        Ok(())
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

    fn resolve_program_text(program_text: String) -> (HashMap<ExprId, usize>, Vec<Stmt>) {
        let scanner = Scanner::from_string(program_text);

        let (tokens, _) = scanner.tokenize();

        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        let mut resolver = Resolver::new();
        (resolver.resolve(&program).unwrap(), program)
    }

    #[test]
    pub fn test_resolver_simple_block() {
        let program_text = "
        var a=3;
        {
          print a;
        }
        print a;
        "
        .to_string();

        let (locals, program) = resolve_program_text(program_text);

        println!("{program:?}");
        println!("{locals:?}");

        // Both references are global, so neither is resolved as a local.
        assert!(!locals.contains_key(&ExprId(1)));
        assert!(!locals.contains_key(&ExprId(2)));
    }

    #[test]
    pub fn test_resolver_shadow_inside_simple_block() {
        let program_text = "
        var a=3;
        {
          var a=4;
          print a;
        }
        print a;
        "
        .to_string();

        let (locals, program) = resolve_program_text(program_text);

        println!("{program:?}");
        println!("{locals:?}");

        // The first print refers to the local a in the block.
        assert_eq!(locals[&ExprId(2)], 0);
        // The second print refers to the global a.
        assert!(!locals.contains_key(&ExprId(3)));
    }

    #[test]
    pub fn test_resolver_global_from_function() {
        let program_text = "
        var a=3;

        fun showA(b){
          print a;
          var a=5;
          print a;
          print b;
        }

        showA(42.0);
        "
        .to_string();

        let (locals, program) = resolve_program_text(program_text);

        println!("{program:?}");
        println!("{locals:?}");

        // The first print a refers to the global a.
        assert!(!locals.contains_key(&ExprId(1)));
        // The second print a inside function refers to shadowing var def just before it
        assert_eq!(locals[&ExprId(3)], 0);
        // The print b inside function refers to the paramter in same scope
        assert_eq!(locals[&ExprId(4)], 0);
    }

    #[test]
    pub fn test_resolver_sample_show_a() {
        let program_text = "
        var a = \"global\";
        {
        fun showA() {
            print a;
        }

        showA();
        var a = \"block\";
        showA();
        }
        "
        .to_string();

        let (locals, program) = resolve_program_text(program_text);

        println!("{program:?}");
        println!("{locals:?}");

        // The function's references to a are global, while the function name
        // references are local to the enclosing block.
        assert!(!locals.contains_key(&ExprId(1)));
        assert_eq!(locals[&ExprId(2)], 0);
        assert_eq!(locals[&ExprId(5)], 0);
    }

    #[test]
    pub fn test_resolver_local_variable_in_own_initializer() {
        let scanner = Scanner::from_string("{ var a = a; }".to_string());
        let (tokens, _) = scanner.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        let mut resolver = Resolver::new();
        assert!(matches!(
            resolver.resolve(&program),
            Err(LoxError::SyntaxError(_))
        ));
    }
}
