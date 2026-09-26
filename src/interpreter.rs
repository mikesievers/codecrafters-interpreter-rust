use crate::{
    environment::Environment,
    expr::Expr,
    lox_error::LoxError,
    lox_function::{CLOCK, LoxFunction},
    lox_value::LoxValue,
    stmt::Stmt,
    token::{TokenType, TokenValue},
};

pub struct Interpreter<'a> {
    globals: Environment<'a>,
    env: Option<Environment<'a>>,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        // Create global environment with builtin functions
        let mut globals = Environment::new();
        globals.put("clock", Some(LoxValue::BuiltinFunction(CLOCK)));

        let env = globals.clone();

        Interpreter {
            globals,
            env: Some(env),
        }
    }

    pub fn interpret(&mut self, program: &[Stmt]) -> Result<(), LoxError> {
        for (idx, stmt) in program.iter().enumerate() {
            match self.execute(stmt) {
                Ok(()) => (),
                Err(LoxError::RuntimeError(e)) => {
                    eprintln!("{e}");
                    eprintln!("[line {}]", idx + 1);
                    return Err(LoxError::RuntimeError(e));
                }
                Err(LoxError::SyntaxError(e)) => {
                    eprintln!("{e}");
                    eprintln!("[line {}]", idx + 1);
                    return Err(LoxError::SyntaxError(e));
                }
            }
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), LoxError> {
        match stmt {
            Stmt::Expression(expr) => match self.evaluate(expr) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            Stmt::Print(expr) => {
                println!("{}", self.evaluate(expr)?);
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                if let Some(expr) = initializer {
                    let value = self.evaluate(expr)?;
                    self.env
                        .as_mut()
                        .expect("Interpreter must have an Environment")
                        .put(*name, Some(value));
                    Ok(())
                } else {
                    self.env
                        .as_mut()
                        .expect("Interpreter must have an Environment")
                        .put(*name, None);
                    Ok(())
                }
            }
            Stmt::Block(stmts) => {
                // Create new env
                // making the old env the enclosing env
                let outer_env = self
                    .env
                    .take()
                    .expect("Interpreter must have an Environment");
                self.env = Some(Environment::new_enclosed(outer_env));
                // Execute statements
                for stmt in stmts {
                    self.execute(stmt)?;
                }
                // recreate old env
                let enclosing = self
                    .env
                    .as_mut()
                    .expect("Interpreter must have an Environment")
                    .take_enclosing()
                    .expect(
                        "At end of block, the environment from the start must still be present",
                    );
                self.env = Some(*enclosing);
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
                Ok(())
            }
            Stmt::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    self.execute(body)?;
                }
                Ok(())
            }
            Stmt::Function { name, params, body } => {
                let environment = self.globals.clone();

                todo!()
            }
        }
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<LoxValue, LoxError> {
        match expr {
            Expr::Literal(token_value) => match token_value {
                TokenValue::String(s) => Ok(LoxValue::String(s.to_string())),
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
                    .assign(*name, Some(evaluated_value))?;
                self.evaluate(&Expr::Variable(name))
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
                    LoxValue::BuiltinFunction(builtin_function) => builtin_function,
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
                lexeme: "-",
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
                lexeme: "*",
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
                lexeme: "/",
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
                lexeme: "+",
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
        let left = Box::new(Expr::Literal(TokenValue::String("4")));
        let right = Box::new(Expr::Literal(TokenValue::String("2")));
        let sub = Expr::Binary {
            operator: Token {
                token_type: TokenType::Plus,
                lexeme: "+",
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
                lexeme: ">",
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
                lexeme: "==",
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
