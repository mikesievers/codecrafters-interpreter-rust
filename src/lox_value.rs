use std::fmt::Display;

// The LoxValue is almost identical to the TokenValue
// The exception is the String - while the TokenValue
// is expected not to duplicate String and refer to the
// program text instead, a LoxValue is expcted to hold
// e.g. a concatenation of Strings.
#[derive(Debug, Clone, PartialEq)]
pub enum LoxValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            LoxValue::String(s) => { s.clone()},
            LoxValue::Number(n) => format!("{n}"),
            LoxValue::Boolean(b) => b.to_string(),
            LoxValue::Nil => "nil".to_string(),
        };
        write!(f, "{output}")
    }
}