use std::fs::read_to_string;

use anyhow::Result;

pub struct Scanner {
    data: String,
}

impl Scanner {
    pub fn from_string(data: String) -> Self {
        Scanner { data }
    }

    pub fn from_file(filename: &String) -> Result<Self> {
        let data = read_to_string(filename)?;
        Ok(Scanner { data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner() {
        let scanner = Scanner::from_string("(()".into());
    }
}
