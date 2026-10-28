use std::{
    fmt::Display,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    environment::{EnvRef, Environment},
    interpreter::Interpreter,
    lox_class::Class,
    lox_error::LoxError,
    lox_value::LoxValue,
    stmt::Stmt,
};

#[derive(Debug, Clone)]
pub enum Callable {
    BuiltinFunction {
        name: &'static str,
        n_args: usize,
        call: fn(&mut Interpreter, Vec<LoxValue>) -> Result<LoxValue, LoxError>,
    },
    UserFunction {
        name: String,
        closure: Option<EnvRef>,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Class(Rc<Class>),
}

impl Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::BuiltinFunction { name, .. } => {
                write!(f, "{name}")
            }
            Callable::UserFunction { name, .. } => write!(f, "{name}"),
            Callable::Class(class) => write!(f, "{}", class.as_ref().name),
        }
    }
}

impl Callable {
    pub fn arity(&self) -> usize {
        match self {
            Callable::BuiltinFunction { n_args, .. } => *n_args,
            Callable::UserFunction { params, .. } => params.len(),
            Callable::Class(class) => 0,
        }
    }
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxError> {
        match self {
            Callable::BuiltinFunction { call, .. } => call(interpreter, arguments),
            Callable::UserFunction {
                params,
                body,
                closure,
                ..
            } => {
                // Create a fresh environment for this call, enclosing the closure.
                // Each call must get its own environment so that recursive calls
                // don't clobber each other's parameters (e.g. `n` in fib).
                let call_env = match closure {
                    Some(closure) => Environment::new_child(closure),
                    None => Environment::new_local(),
                };
                // Store the parameters in the new environment
                for (name, value) in params.iter().zip(arguments.iter()) {
                    call_env.borrow_mut().put(name, Some(value.clone()));
                }
                match interpreter.execute(&Stmt::Block(body.clone()), Some(call_env)) {
                    crate::interpreter::Signal::Ok(lox_value)
                    | crate::interpreter::Signal::Return(lox_value) => Ok(lox_value),
                    crate::interpreter::Signal::Err(lox_error) => Err(lox_error),
                }
            }
            Callable::Class(class) => todo!("Crate an instance with an Rc reference to the class"),
        }
    }
}

// Built in functions

// clock()
pub const CLOCK: Callable = Callable::BuiltinFunction {
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
