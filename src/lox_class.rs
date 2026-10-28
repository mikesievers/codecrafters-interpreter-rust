use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.as_str())
    }
}

impl Class {
    pub fn new(name: &String) -> Self {
        Class { name: name.clone() }
    }
}
