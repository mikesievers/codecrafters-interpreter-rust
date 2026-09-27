use itertools::Itertools;

use crate::{
    environment::{EnvRef, Environment},
    expr::Expr,
    lox_error::LoxError,
    lox_function::{CLOCK, Function},
    lox_value::LoxValue,
    stmt::Stmt,
    token::{TokenType, TokenValue},
};

pub struct Interpreter {
    env: Option<EnvRef>,
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
    pub fn new() -> Self {
        // Create global environment with builtin functions
        let globals = Environment::new_global();
        globals
            .borrow_mut()
            .put("clock", Some(LoxValue::Function(CLOCK)));

        let env = globals.clone();

        Interpreter { env: Some(env) }
    }

    pub fn get_env_clone(&self) -> Result<EnvRef, LoxError> {
        match &self.env {
            Some(env) => Ok(env.clone()),
            None => Err(LoxError::RuntimeError("Interpreter is missing env.".into())),
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
    pub fn execute(
        &mut self,
        stmt: &Stmt,
        mut new_enclosing_environment: Option<EnvRef>,
    ) -> Signal {
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
                    self.env
                        .as_mut()
                        .expect("Interpreter must have an Environment")
                        .borrow_mut()
                        .put(name.clone(), Some(value));
                    Signal::Ok(LoxValue::Nil)
                } else {
                    self.env
                        .as_mut()
                        .expect("Interpreter must have an Environment")
                        .borrow_mut()
                        .put(name.clone(), None);
                    Signal::Ok(LoxValue::Nil)
                }
            }
            Stmt::Block(stmts) => {
                // Create new env for the block
                // If an enclosing env has been specified (i.e. for enclosures),
                // use that. Otherwise use the current enclosing/outer env
                let outer_env = self
                    .env
                    .take()
                    .expect("Interpreter must have an Environment");
                self.env = Some(Environment::new_child(
                    &new_enclosing_environment.unwrap_or(outer_env.clone()),
                ));
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
                self.env = Some(outer_env);
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
                let env = self
                    .env
                    .as_mut()
                    .expect("When defining a function, should always exist.");

                let closure = env.clone();

                env.borrow_mut().put(
                    &name.lexeme,
                    Some(LoxValue::Function(Function::UserFunction {
                        name: name.lexeme.clone(),
                        closure: Environment::new_child(&closure),
                        params: params
                            .iter()
                            .map(|token| token.lexeme.clone())
                            .collect_vec(),
                        // body is a Box and needs dereferencing
                        body: match &**body {
                            Stmt::Block(stmts) => stmts.clone(),
                            _ => return Signal::Err(LoxError::RuntimeError(
                                "Encountered a function with something other then a Block as Stmt."
                                    .into(),
                            )),
                        },
                    })),
                );
                Signal::Ok(LoxValue::Nil)
            }
            Stmt::Return { keyword, value } => match value {
                Some(e) => match self.evaluate(e) {
                    Ok(v) => Signal::Return(v),
                    Err(e) => Signal::Err(e),
                },
                None => Signal::Return(LoxValue::Nil),
            },
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn evaluate(&mut self, expr: &Expr) -> Result<LoxValue, LoxError> {
        match expr {
            Expr::Literal(token_value) => match token_value {
                TokenValue::String(s) => Ok(LoxValue::String(s.clone())),
                TokenValue::Number(n) => Ok(LoxValue::Number(*n)),
                TokenValue::Boolean(b) => Ok(LoxValue::Boolean(*b)),
                TokenValue::Nil => Ok(LoxValue::Nil),
            },
            Expr::Grouping(grp) => self.evaluate(grp),
            Expr::Unary { operator, right } => match operator.token_type {
                TokenType::Minus => Ok((-self.evaluate(right)?)?),
                TokenType::Bang => Ok(!self.evaluate(right)?),
                _ => Err(LoxError::SyntaxError(
                    "Unexpected Unary Operator found".to_string(),
                )),
            },
            Expr::Binary {
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
            Expr::Variable(name) => {
                match self
                    .env
                    .as_ref()
                    .expect("Interpreter must have an env")
                    .borrow()
                    .get(name)?
                {
                    Some(val) => Ok(val),
                    None => Ok(LoxValue::Nil),
                }
            }
            Expr::Assign { name, value } => {
                let evaluated_value = self.evaluate(value)?;
                self.env
                    .as_mut()
                    .expect("Interpreter must have an env")
                    .borrow_mut()
                    .assign(name.clone(), Some(evaluated_value))?;
                self.evaluate(&Expr::Variable(name.clone()))
            }
            Expr::Logical {
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
            Expr::Call {
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
    use crate::token::Token;

    // Evaluation
    #[test]
    fn test_eval_literal() {
        let mut interpreter = Interpreter::new();
        let expr = Expr::Literal(TokenValue::Boolean(true));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            LoxValue::Boolean(true)
        );
    }

    #[test]
    fn test_minus_number() {
        let left = Box::new(Expr::Literal(TokenValue::Number(44.0)));
        let right = Box::new(Expr::Literal(TokenValue::Number(2.0)));
        let sub = Expr::Binary {
            operator: Token {
                token_type: TokenType::Minus,
                lexeme: "-".to_string(),
                literal: None,
            },
            left,
            right,
        };
        let mut interpreter = Interpreter::new();
        assert_eq!(interpreter.evaluate(&sub).unwrap(), LoxValue::Number(42.0));
    }

    #[test]
    fn test_star_number() {
        let left = Box::new(Expr::Literal(TokenValue::Number(4.0)));
        let right = Box::new(Expr::Literal(TokenValue::Number(2.0)));
        let sub = Expr::Binary {
            operator: Token {
                token_type: TokenType::Star,
                lexeme: "*".to_string(),
                literal: None,
            },
            left,
            right,
        };

        let mut interpreter = Interpreter::new();
        assert_eq!(interpreter.evaluate(&sub).unwrap(), LoxValue::Number(8.0));
    }

    #[test]
    fn test_slash_number() {
        let left = Box::new(Expr::Literal(TokenValue::Number(4.0)));
        let right = Box::new(Expr::Literal(TokenValue::Number(2.0)));
        let sub = Expr::Binary {
            operator: Token {
                token_type: TokenType::Slash,
                lexeme: "/".to_string(),
                literal: None,
            },
            left,
            right,
        };

        let mut interpreter = Interpreter::new();
        assert_eq!(interpreter.evaluate(&sub).unwrap(), LoxValue::Number(2.0));
    }

    #[test]
    fn test_add_number() {
        let left = Box::new(Expr::Literal(TokenValue::Number(40.0)));
        let right = Box::new(Expr::Literal(TokenValue::Number(2.0)));
        let sub = Expr::Binary {
            operator: Token {
                token_type: TokenType::Plus,
                lexeme: "+".to_string(),
                literal: None,
            },
            left,
            right,
        };

        let mut interpreter = Interpreter::new();
        assert_eq!(interpreter.evaluate(&sub).unwrap(), LoxValue::Number(42.0));
    }

    #[test]
    fn test_add_string() {
        let left = Box::new(Expr::Literal(TokenValue::String("4".into())));
        let right = Box::new(Expr::Literal(TokenValue::String("2".into())));
        let sub = Expr::Binary {
            operator: Token {
                token_type: TokenType::Plus,
                lexeme: "+".to_string(),
                literal: None,
            },
            left,
            right,
        };

        let mut interpreter = Interpreter::new();
        assert_eq!(
            interpreter.evaluate(&sub).unwrap(),
            LoxValue::String("42".to_string())
        );
    }

    #[test]
    fn test_greater() {
        let left = Box::new(Expr::Literal(TokenValue::Number(4.0)));
        let right = Box::new(Expr::Literal(TokenValue::Number(2.0)));
        let sub = Expr::Binary {
            operator: Token {
                token_type: TokenType::Greater,
                lexeme: ">".to_string(),
                literal: None,
            },
            left,
            right,
        };

        let mut interpreter = Interpreter::new();
        assert_eq!(interpreter.evaluate(&sub).unwrap(), LoxValue::Boolean(true));
    }

    #[test]
    fn test_equal_equal() {
        let left = Box::new(Expr::Literal(TokenValue::Number(4.0)));
        let right = Box::new(Expr::Literal(TokenValue::Number(2.0)));
        let sub = Expr::Binary {
            operator: Token {
                token_type: TokenType::EqualEqual,
                lexeme: "==".to_string(),
                literal: None,
            },
            left,
            right,
        };

        let mut interpreter = Interpreter::new();
        assert_eq!(
            interpreter.evaluate(&sub).unwrap(),
            LoxValue::Boolean(false)
        );
    }
}
