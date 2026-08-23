use std::fmt::Display;


#[derive(Debug)]
pub enum LoxError {
    SyntaxError(String),
    RuntimeError(String),
}

impl Display for LoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            LoxError::SyntaxError(e) | LoxError::RuntimeError(e) => e,
        };
        write!(f, "{output}")
    }
}