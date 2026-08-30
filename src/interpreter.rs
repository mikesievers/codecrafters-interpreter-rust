use crate::{environment::Environment, lox_error::LoxError, stmt::Stmt};

pub struct Interpreter<'a> {
    env: Environment<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        Interpreter {
            env: Environment::new(),
        }
    }

    pub fn interpret(&mut self, program: &[Stmt]) -> Result<(), LoxError> {
        for (idx, stmt) in program.iter().enumerate() {
            match stmt.execute() {
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
}
