use std::collections::HashMap;

use itertools::Itertools;

use crate::{
    environment::{EnvRef, Environment, EnvironmentExt},
    expr::{Expr, ExprId, ExprKind},
    lox_class::Class,
    lox_error::LoxError,
    lox_function::{CLOCK, Function},
    lox_value::LoxValue,
    stmt::Stmt,
    token::{TokenType, TokenValue},
};

#[derive(Debug)]
pub struct Interpreter {
    env: Option<EnvRef>,
    globals: EnvRef,
    locals: HashMap<ExprId, usize>,
}

// Create a Signal to return from evaluation/execution, to allow
// short circuiting for return statements.
// Java can solve this with exceptions to unwind call stacks,
// for Rust an enum looks more idiomatic
pub enum Signal {
    Ok(LoxValue),
    Return(LoxValue),
    Err(LoxError),
}

impl Interpreter {
    pub fn new(locals: HashMap<ExprId, usize>) -> Self {
        // Create global environment with builtin functions
        let globals = Environment::new_global();
        globals
            .borrow_mut()
            .put("clock", Some(LoxValue::Function(CLOCK)));

        Interpreter {
            env: None,
            globals,
            locals,
        }
    }

    fn define_variable(&mut self, name: &str, value: Option<LoxValue>) {
        match self.env.as_ref() {
            Some(env) => env.borrow_mut().put(name, value),
            None => self.globals.borrow_mut().put(name, value),
        }
    }

    fn lookup_variable(&self, expr_id: ExprId, name: &str) -> Result<Option<LoxValue>, LoxError> {
        match self.locals.get(&expr_id) {
            Some(&distance) => self
                .env
                .as_ref()
                .expect("Must have an env")
                .ancestor(distance)
                .borrow()
                .get(&name),
            None => self.globals.borrow().get(&name),
        }
    }

    fn assign_variable(
        &mut self,
        expr_id: ExprId,
        name: &str,
        value: LoxValue,
    ) -> Result<(), LoxError> {
        match self.locals.get(&expr_id) {
            Some(&distance) => self
                .env
                .as_ref()
                .expect("Must have an env")
                .ancestor(distance)
                .borrow_mut()
                .assign(name, Some(value)),
            None => self.globals.borrow_mut().assign(name, Some(value)),
        }
    }

    pub fn interpret(&mut self, program: &[Stmt]) -> Result<(), LoxError> {
        for (idx, stmt) in program.iter().enumerate() {
            match self.execute(stmt, None) {
                Signal::Ok(_) => (),
                Signal::Err(LoxError::RuntimeError(e)) => {
                    eprintln!("{e}");
                    eprintln!("[line {}]", idx + 1);
                    return Err(LoxError::RuntimeError(e));
                }
                Signal::Err(LoxError::SyntaxError(e)) => {
                    eprintln!("{e}");
                    eprintln!("[line {}]", idx + 1);
                    return Err(LoxError::SyntaxError(e));
                }
                Signal::Return(_) => {
                    eprintln!("Return statement found at global level");
                    return Err(LoxError::RuntimeError(
                        "Global level return statement".into(),
                    ));
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    pub fn execute(&mut self, stmt: &Stmt, new_enclosing_environment: Option<EnvRef>) -> Signal {
        // If an environment is given, execute in that environment rather than the current one
        match stmt {
            Stmt::Expression(expr) => match self.evaluate(expr) {
                Ok(_) => Signal::Ok(LoxValue::Nil),
                Err(e) => Signal::Err(e),
            },
            Stmt::Print(expr) => match self.evaluate(expr) {
                Ok(v) => {
                    println!("{v}");
                    Signal::Ok(LoxValue::Nil)
                }
                Err(e) => Signal::Err(e),
            },
            Stmt::Var { name, initializer } => {
                if let Some(expr) = initializer {
                    let value = match self.evaluate(expr) {
                        Ok(v) => v,
                        Err(e) => return Signal::Err(e),
                    };
                    self.define_variable(name, Some(value));
                    Signal::Ok(LoxValue::Nil)
                } else {
                    self.define_variable(name, None);
                    Signal::Ok(LoxValue::Nil)
                }
            }
            Stmt::Block(stmts) => {
                // Create new env for the block
                // If an enclosing env has been specified (i.e. for enclosures),
                // use that. Otherwise use the current enclosing/outer env
                let outer_env = self.env.take();
                self.env = match new_enclosing_environment {
                    Some(closure) => Some(closure.clone()),
                    None => Some(match &outer_env {
                        Some(outer_env) => Environment::new_child(outer_env),
                        None => Environment::new_local(),
                    }),
                };
                // Execute statements, short-circuiting on Return/Err
                let result = {
                    let mut short_circuit: Option<Signal> = None;
                    for stmt in stmts {
                        match self.execute(stmt, None) {
                            Signal::Ok(_) => {}
                            Signal::Return(lox_value) => {
                                short_circuit = Some(Signal::Return(lox_value));
                                break;
                            }
                            Signal::Err(lox_error) => {
                                short_circuit = Some(Signal::Err(lox_error));
                                break;
                            }
                        }
                    }
                    short_circuit.unwrap_or(Signal::Ok(LoxValue::Nil))
                };
                // recreate old env (always, even when short-circuited by a return)
                self.env = outer_env;
                result
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let expr_result = match self.evaluate(condition) {
                    Ok(v) => v,
                    Err(e) => return Signal::Err(e),
                };
                if expr_result.is_truthy() {
                    match self.execute(then_branch, None) {
                        Signal::Ok(lox_value) => Signal::Ok(lox_value),
                        Signal::Return(lox_value) => return Signal::Return(lox_value),
                        Signal::Err(lox_error) => return Signal::Err(lox_error),
                    };
                } else if let Some(else_branch) = else_branch {
                    match self.execute(else_branch, None) {
                        Signal::Ok(lox_value) => Signal::Ok(lox_value),
                        Signal::Return(lox_value) => return Signal::Return(lox_value),
                        Signal::Err(lox_error) => return Signal::Err(lox_error),
                    };
                }
                Signal::Ok(LoxValue::Nil)
            }
            Stmt::While { condition, body } => {
                while match self.evaluate(condition) {
                    Ok(v) => v.is_truthy(),
                    Err(e) => return Signal::Err(e),
                } {
                    match self.execute(body, None) {
                        Signal::Ok(lox_value) => Signal::Ok(lox_value),
                        Signal::Return(lox_value) => return Signal::Return(lox_value),
                        Signal::Err(lox_error) => return Signal::Err(lox_error),
                    };
                }
                Signal::Ok(LoxValue::Nil)
            }
            Stmt::Function { name, params, body } => {
                let closure = self.env.clone();
                let function_body = match &**body {
                    Stmt::Block(stmts) => stmts.clone(),
                    _ => {
                        return Signal::Err(LoxError::RuntimeError(
                            "Encountered a function with something other then a Block as Stmt."
                                .into(),
                        ));
                    }
                };
                self.define_variable(
                    &name.lexeme,
                    Some(LoxValue::Function(Function::UserFunction {
                        name: name.lexeme.clone(),
                        closure,
                        params: params
                            .iter()
                            .map(|token| token.lexeme.clone())
                            .collect_vec(),
                        body: function_body,
                    })),
                );
                Signal::Ok(LoxValue::Nil)
            }
            Stmt::Return { value, .. } => match value {
                Some(e) => match self.evaluate(e) {
                    Ok(v) => Signal::Return(v),
                    Err(e) => Signal::Err(e),
                },
                None => Signal::Return(LoxValue::Nil),
            },
            Stmt::Class { id, name, .. } => {
                self.define_variable(&name.lexeme, None);
                let class = Class::new(&name.lexeme);
                // TODO: The expression ID should get a real meaning
                // Using the next value as a workaround until it causes a
                // problem that needs fixing
                self.assign_variable(*id, &name.lexeme, LoxValue::Class(class));

                Signal::Ok(LoxValue::Nil)
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn evaluate(&mut self, expr: &Expr) -> Result<LoxValue, LoxError> {
        match &expr.kind {
            ExprKind::Literal(token_value) => match token_value {
                TokenValue::String(s) => Ok(LoxValue::String(s.clone())),
                TokenValue::Number(n) => Ok(LoxValue::Number(*n)),
                TokenValue::Boolean(b) => Ok(LoxValue::Boolean(*b)),
                TokenValue::Nil => Ok(LoxValue::Nil),
            },
            ExprKind::Grouping(grp) => self.evaluate(grp),
            ExprKind::Unary { operator, right } => match operator.token_type {
                TokenType::Minus => Ok((-self.evaluate(right)?)?),
                TokenType::Bang => Ok(!self.evaluate(right)?),
                _ => Err(LoxError::SyntaxError(
                    "Unexpected Unary Operator found".to_string(),
                )),
            },
            ExprKind::Binary {
                operator,
                left,
                right,
            } => match operator.token_type {
                TokenType::Minus => Ok((self.evaluate(left)? - self.evaluate(right)?)?),
                TokenType::Plus => Ok((self.evaluate(left)? + self.evaluate(right)?)?),
                TokenType::Star => Ok((self.evaluate(left)? * self.evaluate(right)?)?),
                TokenType::Slash => Ok((self.evaluate(left)? / self.evaluate(right)?)?),
                TokenType::EqualEqual => Ok(LoxValue::Boolean(
                    self.evaluate(left)? == self.evaluate(right)?,
                )),
                TokenType::BangEqual => Ok(LoxValue::Boolean(
                    self.evaluate(left)? != self.evaluate(right)?,
                )),
                TokenType::Greater => {
                    compare_if_numbers(">", self.evaluate(left)?, self.evaluate(right)?)
                }
                TokenType::GreaterEqual => {
                    compare_if_numbers(">=", self.evaluate(left)?, self.evaluate(right)?)
                }
                TokenType::Less => {
                    compare_if_numbers("<", self.evaluate(left)?, self.evaluate(right)?)
                }
                TokenType::LessEqual => {
                    compare_if_numbers("<=", self.evaluate(left)?, self.evaluate(right)?)
                }

                _ => Err(LoxError::RuntimeError(
                    "Unexpected Binary operator encountered".to_string(),
                )),
            },
            ExprKind::Variable(name) => match self.lookup_variable(expr.id, name)? {
                Some(val) => Ok(val),
                None => Ok(LoxValue::Nil),
            },
            ExprKind::Assign { name, value } => {
                let evaluated_value = self.evaluate(value)?;
                self.assign_variable(expr.id, name, evaluated_value.clone())?;
                Ok(evaluated_value)
            }
            ExprKind::Logical {
                left,
                operator,
                right,
            } => {
                let left_result = self.evaluate(left)?;
                if operator.token_type == TokenType::Or {
                    if left_result.is_truthy() {
                        return Ok(left_result);
                    }
                } else if !left_result.is_truthy() {
                    return Ok(left_result);
                }
                self.evaluate(right)
            }
            ExprKind::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = match self.evaluate(callee)? {
                    LoxValue::Function(function) => function,
                    _ => {
                        return Err(LoxError::RuntimeError(
                            "Callee is not a function".to_string(),
                        ));
                    }
                };
                let arguments = arguments
                    .iter()
                    .map(|arg| self.evaluate(arg))
                    .collect::<Result<Vec<_>, _>>()?;

                if arguments.len() != callee.arity() {
                    return Err(LoxError::RuntimeError(
                        "Wrong number of arguments for function".to_string(),
                    ));
                }

                callee.call(self, arguments)
            }
        }
    }
}

fn compare_if_numbers(
    operator: &str,
    left: LoxValue,
    right: LoxValue,
) -> Result<LoxValue, LoxError> {
    match (operator, left, right) {
        (">", LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Boolean(l > r)),
        (">=", LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Boolean(l >= r)),
        ("<", LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Boolean(l < r)),
        ("<=", LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Boolean(l <= r)),
        _ => Err(LoxError::RuntimeError(
            "Operands should be numbers.".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Scanner, expr::ExprId, parser::Parser, resolver::Resolver, token::Token};

    // Evaluation
    #[test]
    fn test_eval_literal() {
        let mut interpreter = Interpreter::new(HashMap::new());
        let expr = Expr {
            id: ExprId(0),
            kind: ExprKind::Literal(TokenValue::Boolean(true)),
        };
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            LoxValue::Boolean(true)
        );
    }

    #[test]
    fn test_minus_number() {
        let left = Box::new(Expr {
            id: ExprId(0),
            kind: ExprKind::Literal(TokenValue::Number(44.0)),
        });
        let right = Box::new(Expr {
            id: ExprId(1),
            kind: ExprKind::Literal(TokenValue::Number(2.0)),
        });
        let sub = Expr {
            id: ExprId(2),
            kind: ExprKind::Binary {
                operator: Token {
                    token_type: TokenType::Minus,
                    lexeme: "-".to_string(),
                    literal: None,
                },
                left,
                right,
            },
        };
        let mut interpreter = Interpreter::new(HashMap::new());
        assert_eq!(interpreter.evaluate(&sub).unwrap(), LoxValue::Number(42.0));
    }

    #[test]
    fn test_star_number() {
        let left = Box::new(Expr {
            id: ExprId(0),
            kind: ExprKind::Literal(TokenValue::Number(4.0)),
        });
        let right = Box::new(Expr {
            id: ExprId(1),
            kind: ExprKind::Literal(TokenValue::Number(2.0)),
        });
        let sub = Expr {
            id: ExprId(2),
            kind: ExprKind::Binary {
                operator: Token {
                    token_type: TokenType::Star,
                    lexeme: "*".to_string(),
                    literal: None,
                },
                left,
                right,
            },
        };

        let mut interpreter = Interpreter::new(HashMap::new());
        assert_eq!(interpreter.evaluate(&sub).unwrap(), LoxValue::Number(8.0));
    }

    #[test]
    fn test_slash_number() {
        let left = Box::new(Expr {
            id: ExprId(0),
            kind: ExprKind::Literal(TokenValue::Number(4.0)),
        });
        let right = Box::new(Expr {
            id: ExprId(1),
            kind: ExprKind::Literal(TokenValue::Number(2.0)),
        });
        let sub = Expr {
            id: ExprId(2),
            kind: ExprKind::Binary {
                operator: Token {
                    token_type: TokenType::Slash,
                    lexeme: "/".to_string(),
                    literal: None,
                },
                left,
                right,
            },
        };

        let mut interpreter = Interpreter::new(HashMap::new());
        assert_eq!(interpreter.evaluate(&sub).unwrap(), LoxValue::Number(2.0));
    }

    #[test]
    fn test_add_number() {
        let left = Box::new(Expr {
            id: ExprId(0),
            kind: ExprKind::Literal(TokenValue::Number(40.0)),
        });
        let right = Box::new(Expr {
            id: ExprId(1),
            kind: ExprKind::Literal(TokenValue::Number(2.0)),
        });
        let sub = Expr {
            id: ExprId(2),
            kind: ExprKind::Binary {
                operator: Token {
                    token_type: TokenType::Plus,
                    lexeme: "+".to_string(),
                    literal: None,
                },
                left,
                right,
            },
        };

        let mut interpreter = Interpreter::new(HashMap::new());
        assert_eq!(interpreter.evaluate(&sub).unwrap(), LoxValue::Number(42.0));
    }

    #[test]
    fn test_add_string() {
        let left = Box::new(Expr {
            id: ExprId(0),
            kind: ExprKind::Literal(TokenValue::String("4".into())),
        });
        let right = Box::new(Expr {
            id: ExprId(0),
            kind: ExprKind::Literal(TokenValue::String("2".into())),
        });
        let sub = Expr {
            id: ExprId(0),
            kind: ExprKind::Binary {
                operator: Token {
                    token_type: TokenType::Plus,
                    lexeme: "+".to_string(),
                    literal: None,
                },
                left,
                right,
            },
        };

        let mut interpreter = Interpreter::new(HashMap::new());
        assert_eq!(
            interpreter.evaluate(&sub).unwrap(),
            LoxValue::String("42".to_string())
        );
    }

    #[test]
    fn test_greater() {
        let left = Box::new(Expr {
            id: ExprId(0),
            kind: ExprKind::Literal(TokenValue::Number(4.0)),
        });
        let right = Box::new(Expr {
            id: ExprId(0),
            kind: ExprKind::Literal(TokenValue::Number(2.0)),
        });
        let sub = Expr {
            id: ExprId(0),
            kind: ExprKind::Binary {
                operator: Token {
                    token_type: TokenType::Greater,
                    lexeme: ">".to_string(),
                    literal: None,
                },
                left,
                right,
            },
        };

        let mut interpreter = Interpreter::new(HashMap::new());
        assert_eq!(interpreter.evaluate(&sub).unwrap(), LoxValue::Boolean(true));
    }

    #[test]
    fn test_equal_equal() {
        let left = Box::new(Expr {
            id: ExprId(0),
            kind: ExprKind::Literal(TokenValue::Number(4.0)),
        });
        let right = Box::new(Expr {
            id: ExprId(0),
            kind: ExprKind::Literal(TokenValue::Number(2.0)),
        });
        let sub = Expr {
            id: ExprId(0),
            kind: ExprKind::Binary {
                operator: Token {
                    token_type: TokenType::EqualEqual,
                    lexeme: "==".to_string(),
                    literal: None,
                },
                left,
                right,
            },
        };

        let mut interpreter = Interpreter::new(HashMap::new());
        assert_eq!(
            interpreter.evaluate(&sub).unwrap(),
            LoxValue::Boolean(false)
        );
    }

    // Tests for the interpreter with resolution pass
    fn interpreter_from_program_text(program_text: String) -> (Interpreter, Vec<Stmt>) {
        let scanner = Scanner::from_string(program_text);

        let (tokens, _) = scanner.tokenize();

        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        let mut resolver = Resolver::new();
        let locals = resolver.resolve(&program).unwrap();

        (Interpreter::new(locals), program)
    }

    #[test]
    pub fn test_globals_are_separate_from_local_scopes() {
        let program_text = "
            var a = 1;
            {
                var a = 2;
                a = 3;
            }
        "
        .to_string();

        let (mut interpreter, program) = interpreter_from_program_text(program_text);
        interpreter.interpret(&program).unwrap();

        assert_eq!(
            interpreter.globals.borrow().get(&"a").unwrap(),
            Some(LoxValue::Number(1.0))
        );
        assert!(interpreter.env.is_none());
    }

    #[test]
    pub fn test_functions_use_global_fallback_and_capture_locals() {
        let program_text = "
            var global = 1;
            var result = nil;
            {
                var captured = 2;
                fun update() {
                    captured = 3;
                    global = 4;
                    result = captured;
                }
                update();
            }
        "
        .to_string();

        let (mut interpreter, program) = interpreter_from_program_text(program_text);
        interpreter.interpret(&program).unwrap();

        assert_eq!(
            interpreter.globals.borrow().get(&"global").unwrap(),
            Some(LoxValue::Number(4.0))
        );
        assert_eq!(
            interpreter.globals.borrow().get(&"result").unwrap(),
            Some(LoxValue::Number(3.0))
        );
    }

    #[test]
    pub fn test_top_level_functions_and_builtins_are_global() {
        let program_text = "
            var value = clock();
            fun topLevel() { return value; }
            var result = topLevel();
        "
        .to_string();

        let (mut interpreter, program) = interpreter_from_program_text(program_text);
        interpreter.interpret(&program).unwrap();

        assert!(matches!(
            interpreter.globals.borrow().get(&"value").unwrap(),
            Some(LoxValue::Number(_))
        ));
        assert_eq!(
            interpreter.globals.borrow().get(&"result").unwrap(),
            interpreter.globals.borrow().get(&"value").unwrap()
        );
        let Some(LoxValue::Function(Function::UserFunction { closure, .. })) =
            interpreter.globals.borrow().get(&"topLevel").unwrap()
        else {
            panic!("topLevel should be a user function");
        };
        assert!(closure.is_none());
    }

    #[test]
    pub fn test_unknown_variable_still_errors() {
        let (mut interpreter, program) =
            interpreter_from_program_text("print missing;".to_string());

        assert!(matches!(
            interpreter.interpret(&program),
            Err(LoxError::RuntimeError(message)) if message == "Unknown variable: missing"
        ));
    }
}
