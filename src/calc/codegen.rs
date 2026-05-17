use crate::calc::parser::AST;
use std::collections::HashMap;

pub struct Symbol {
    pub name: String,
    pub value: u8,
}

pub struct Codegen {
    pub code: String,
    pub symbols: HashMap<u8, Symbol>,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
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
        }
    }

    pub fn emit_setup(&mut self, tree: &AST) {
        self.code.push_str("setup\n");
        self.generate_symbols(tree);

        for symbol in self.symbols.values() {
            self.code
                .push_str(&format!("\t{} DATA {}\n", symbol.name, symbol.value));
        }

        self.code.push_str("end\n");
    }
}

impl Default for Codegen {
    fn default() -> Self {
        Self::new()
    }
}
