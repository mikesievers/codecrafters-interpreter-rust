use std::time::{SystemTime, UNIX_EPOCH};

use crate::{interpreter::Interpreter, lox_error::LoxError, lox_value::LoxValue, stmt::Stmt};

pub trait LoxFunction {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxError>;
}

#[derive(Debug, Clone)]
pub struct BuiltinFunction<'a> {
    n_args: usize,
    call: fn(&mut Interpreter, Vec<LoxValue>) -> Result<LoxValue<'a>, LoxError>,
}

impl<'a> LoxFunction for BuiltinFunction<'a> {
    fn arity(&self) -> usize {
        self.n_args
    }
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxError> {
        (self.call)(interpreter, arguments)
    }
}

#[derive(Debug, Clone)]
pub struct Function<'a> {
    pub declaration: Stmt<'a>,
}

impl<'a> LoxFunction for Function<'a> {
    fn arity(&self) -> usize {
        match &self.declaration {
            Stmt::Function { params, .. } => params.len(),
            _ => {
                panic!("A Function should only ever have a Stmt::Function as declaration.")
            }
        }
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxError> {
        todo!()
    }
}

// Built in functions

// clock()
pub const CLOCK: BuiltinFunction = BuiltinFunction {
    n_args: 0,
    call: |_interpreter, _arguments| {
        // Ignore the warning of converting from u128 to f64, the epoch is not
        // supposed to reach that size anytime soon
        #[allow(clippy::cast_precision_loss)]
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time should never go backwards")
            .as_millis() as f64
            / 1000.0;
        Ok(LoxValue::Number(secs))
    },
};
