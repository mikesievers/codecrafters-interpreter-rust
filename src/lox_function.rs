use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{interpreter::Interpreter, lox_error::LoxError, lox_value::LoxValue, stmt::Stmt};

#[derive(Debug, Clone)]
pub enum Function {
    BuiltinFunction {
        name: &'static str,
        n_args: usize,
        call: fn(&mut Interpreter, Vec<LoxValue>) -> Result<LoxValue, LoxError>,
    },
    UserFunction {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::BuiltinFunction { name, .. } => {
                write!(f, "<fn {}>", name)
            }
            Function::UserFunction { name, .. } => write!(f, "<fn {}>", name),
        }
    }
}

impl Function {
    pub fn arity(&self) -> usize {
        match self {
            Function::BuiltinFunction { n_args, .. } => *n_args,
            Function::UserFunction { params, .. } => params.len(),
        }
    }
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxError> {
        match self {
            Function::BuiltinFunction { call, .. } => call(interpreter, arguments),
            Function::UserFunction { name, params, body } => todo!(),
        }
    }
}

// Built in functions

// clock()
pub const CLOCK: Function = Function::BuiltinFunction {
    name: "clock",
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
