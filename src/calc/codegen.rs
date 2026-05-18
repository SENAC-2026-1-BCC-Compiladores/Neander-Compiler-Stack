use crate::calc::parser::AST;
use std::collections::HashMap;

pub struct Symbol {
    pub name: String,
    pub value: u8,
}

pub struct CodegenCalc {
    pub code: String,
    pub symbols: HashMap<u8, Symbol>,
}

impl CodegenCalc {
    pub fn new() -> Self {
        CodegenCalc {
            code: String::new(),
            symbols: HashMap::new(),
        }
    }

    fn generate_symbols(&mut self, tree: &AST) {
        match tree {
            AST::Number(num) => {
                if !self.symbols.contains_key(num) {
                    let symbol = Symbol {
                        name: format!("CONST{}", *num),
                        value: *num,
                    };

                    self.symbols.insert(*num, symbol);
                }
            }
            AST::Add(left, right) => {
                self.generate_symbols(left);
                self.generate_symbols(right);
            }
            AST::Sub(left, right) => {
                self.generate_symbols(left);
                self.generate_symbols(right);
            }
        }
    }

    fn emit_setup(&mut self, tree: &AST) {
        self.code.push_str("setup\n");
        self.generate_symbols(tree);

        for symbol in self.symbols.values() {
            self.code
                .push_str(&format!("\t{} DATA {}\n", symbol.name, symbol.value));
        }

        self.code.push_str("end\n");
    }

    fn generate_instructions(&mut self, tree: &AST) -> String {
        match tree {
            AST::Number(num) => {
                format!("CONST{}", *num)
            }
            AST::Add(left, right) => {
                let left_result = self.generate_instructions(left);
                let right_result = self.generate_instructions(right);
                let res = "t4".to_string();

                self.code.push_str(&format!("\tlda {}\n", left_result));
                self.code.push_str(&format!("\tadd {}\n", right_result));
                self.code.push_str(&format!("\tsta {}\n", res));

                res
            }
            AST::Sub(left, right) => {
                let left_result = self.generate_instructions(left);
                let right_result = self.generate_instructions(right);
                let res = "t4".to_string();

                self.code.push_str(&format!("\tlda {}\n", left_result));
                self.code.push_str(&format!("\tsub {}\n", right_result));
                self.code.push_str(&format!("\tsta {}\n", res));

                res
            }
        }
    }

    fn emit_text(&mut self, tree: &AST) {
        self.code.push_str("text\n");
        self.generate_instructions(tree);
        self.code.push_str("\thlt\n");
        self.code.push_str("end\n");
    }

    pub fn generate_code(&mut self, tree: &AST) {
        self.emit_setup(tree);
        self.emit_text(tree);
    }
}

impl Default for CodegenCalc {
    fn default() -> Self {
        Self::new()
    }
}
