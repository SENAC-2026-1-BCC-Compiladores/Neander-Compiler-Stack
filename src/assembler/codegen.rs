use crate::assembler::LexerError;
use crate::assembler::parser::{DataDecl, Instruction, Operand, Program, Register};
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
        self.map.insert("t3".to_string(), (254, 1));
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

    fn emit_opcode(&mut self, opcode: u8) {
        self.bin[self.pc as usize] = opcode;
        self.pc += 1;
    }

    fn emit_operand(&mut self, operand: &Operand) -> Result<(), LexerError> {
        let addr = self.resolve_operand(operand)?;
        self.bin[self.pc as usize] = addr;
        self.pc += 1;
        Ok(())
    }

    fn emit_unary(&mut self, opcode: u8, operand: &Operand) -> Result<(), LexerError> {
        self.emit_opcode(opcode);
        self.emit_operand(operand)?;
        Ok(())
    }

    fn expand_sub(&mut self, operand: &Operand) -> Result<(), LexerError> {
        let t0 = Operand::Register(Register::T0);
        let t1 = Operand::Register(Register::T1);
        let t3 = Operand::Register(Register::T3);

        self.emit_unary(16, &t0)?;
        self.emit_unary(32, operand)?;
        self.emit_opcode(96);
        self.emit_unary(48, &t3)?;
        self.emit_unary(16, &t1)?;
        self.emit_unary(32, &t0)?;
        self.emit_unary(48, &t1)?;

        Ok(())
    }

    fn emit_instruction(&mut self, instr: &Instruction) -> Result<(), LexerError> {
        match instr {
            Instruction::Nop => {
                self.emit_opcode(0);
            }
            Instruction::Hlt => {
                self.emit_opcode(240);
            }
            Instruction::Not => {
                self.emit_opcode(96);
            }
            Instruction::Add(op) => {
                self.emit_unary(48, op)?;
            }
            Instruction::Sta(op) => {
                self.emit_unary(16, op)?;
            }
            Instruction::Lda(op) => {
                self.emit_unary(32, op)?;
            }
            Instruction::Or(op) => {
                self.emit_unary(64, op)?;
            }
            Instruction::And(op) => {
                self.emit_unary(80, op)?;
            }
            Instruction::Jmp(op) => {
                self.emit_unary(128, op)?;
            }
            Instruction::Jn(op) => {
                self.emit_unary(144, op)?;
            }
            Instruction::Jz(op) => {
                self.emit_unary(160, op)?;
            }
            Instruction::Sub(op) => {
                self.expand_sub(op)?;
            }
        }

        Ok(())
    }

    fn generate_binary(&mut self, program: &Program) -> Result<(), LexerError> {
        for (addr, value) in self.map.values() {
            self.bin[*addr as usize] = *value;
        }

        for instruction in &program.text {
            self.emit_instruction(instruction)?;
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
