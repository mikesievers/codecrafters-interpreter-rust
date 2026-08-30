use crate::{environment::Environment, evaluate::Evaluate, lox_error::LoxError, stmt::Stmt};

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: Environment::new(),
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
            Stmt::Expression(expr) => match expr.evaluate() {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            Stmt::Print(expr) => {
                println!("{}", expr.evaluate()?);
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                if let Some(expr) = initializer {
                    self.env.put(*name, Some(expr.evaluate()?));
                    Ok(())
                } else {
                    self.env.put(*name, None);
                    Ok(())
                }
            }
        }
    }
}
