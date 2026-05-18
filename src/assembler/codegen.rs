use crate::assembler::LexerError;
use crate::assembler::parser::{DataDecl, Instruction, Operand, Program};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

pub type Item = (u8, u8);

#[derive(Debug)]
pub struct Codegen {
    pub pc: u8,
    pub map: HashMap<String, Item>,
    pub bin: [u8; 256],
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

impl Codegen {
    pub fn new() -> Self {
        Self {
            pc: 0,
            map: HashMap::new(),
            bin: [0; 256],
        }
    }

    fn generate_symbols(&mut self, declarations: &[DataDecl]) -> Result<(), SyntaxError> {
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
                    if self.pc != 0 {
                        return Err(SyntaxError::new(
                            "Semantic error. Program count was already set.".into(),
                        ));
                    }
                    self.pc = *addr;
                }
            }
        }

        self.map.insert("t0".to_string(), (251, 0));
        self.map.insert("t1".to_string(), (252, 0));
        self.map.insert("t2".to_string(), (253, 0));
        self.map.insert("t3".to_string(), (254, 0));
        self.map.insert("t4".to_string(), (255, 0));

        Ok(())
    }

    fn resolve_operand(&self, operand: &Operand) -> Result<u8, LexerError> {
        match operand {
            Operand::Register(reg) => Ok(reg.address()),

            Operand::Symbol(name) => match self.map.get(name) {
                Some(&(addr, _)) => Ok(addr),

                None => Err(LexerError::new(format!(
                    "Semantic error. Var '{}' was not found.",
                    name
                ))),
            },
        }
    }

    fn generate_binary(&mut self, program: &Program) -> Result<(), LexerError> {
        for (addr, value) in self.map.values() {
            self.bin[*addr as usize] = *value;
        }

        for instruction in &program.text {
            let (opcode, opt_var) = match instruction {
                Instruction::Nop => (0u8, None),
                Instruction::Hlt => (240u8, None),
                Instruction::Add(v) => (48u8, Some(v)),
                Instruction::Sta(v) => (16u8, Some(v)),
                Instruction::Lda(v) => (32u8, Some(v)),
                Instruction::Or(v) => (64u8, Some(v)),
                Instruction::And(v) => (80u8, Some(v)),
                Instruction::Not(v) => (96u8, Some(v)),
                Instruction::Jmp(v) => (128u8, Some(v)),
                Instruction::Jn(v) => (144u8, Some(v)),
                Instruction::Jz(v) => (160u8, Some(v)),
            };

            self.bin[self.pc as usize] = opcode;
            self.pc += 1;

            if let Some(operand) = opt_var {
                self.bin[self.pc as usize] = self.resolve_operand(operand)?;
                self.pc += 1;
            }
        }

        Ok(())
    }

    fn transform_bin(&self, bin: &[u8]) -> [u8; 516] {
        let mut out_bin: [u8; 516] = [0; 516];

        out_bin[0..4].copy_from_slice(&[3, 78, 68, 82]);

        for i in 0..256 {
            out_bin[i * 2 + 4] = bin[i];
        }

        out_bin
    }

    pub fn generate(&mut self, program: &Program) -> Result<[u8; 516], Box<dyn Error>> {
        self.generate_symbols(&program.setup)?;
        self.generate_binary(program)?;
        Ok(self.transform_bin(&self.bin))
    }
}

impl Default for Codegen {
    fn default() -> Self {
        Self::new()
    }
}

pub fn print_bin(bin: &[u8]) {
    for chunk in bin.chunks(16) {
        for byte in chunk {
            print!(" {:02}", byte);
        }
        println!();
    }
    println!();
}
