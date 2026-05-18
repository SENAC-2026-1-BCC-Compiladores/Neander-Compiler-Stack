use crate::assembler::parser::DataDecl;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

pub type Item = (u8, u8);

#[derive(Debug)]
pub struct SymbolTable {
    pub program_counter: u8,
    pub map: HashMap<String, Item>,
}

#[derive(Debug, Clone)]
pub struct SyntaxError {
    pub error: String,
}

impl SyntaxError {
    pub fn new(message: String) -> Self {
        SyntaxError { error: message }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Syntax error: {}", self.error)
    }
}

impl Error for SyntaxError {}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            program_counter: 0,
            map: HashMap::new(),
        }
    }

    pub fn build(&mut self, declarations: &[DataDecl]) -> Result<(), SyntaxError> {
        let mut current_addr: u8 = 250;

        for decl in declarations {
            match decl {
                DataDecl::Data(var, value) => {
                    if self.map.contains_key(var) {
                        return Err(SyntaxError::new(format!(
                            "Semantic error. Var '{}' was already declared.",
                            var
                        )));
                    }

                    self.map.insert(var.clone(), (current_addr, *value));
                    current_addr -= 1;
                }
                DataDecl::Space(var, size) => {
                    if self.map.contains_key(var) {
                        return Err(SyntaxError::new(format!(
                            "Semantic error. Var '{}' was already declared.",
                            var
                        )));
                    }

                    current_addr -= size - 1;

                    self.map.insert(var.clone(), (current_addr, 0u8));
                }
                DataDecl::Org(addr) => {
                    if self.program_counter != 0 {
                        return Err(SyntaxError::new(
                            "Semantic error. Program count was already set.".into(),
                        ));
                    }
                    self.program_counter = *addr;
                }
            }
        }

        self.map.insert("t0".to_string(), (251, 0));
        self.map.insert("t2".to_string(), (252, 0));
        self.map.insert("t3".to_string(), (253, 0));
        self.map.insert("t4".to_string(), (254, 0));
        self.map.insert("t5".to_string(), (255, 0));

        Ok(())
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
