use crate::assembler::*;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum DataDecl {
    Data(String, u8),
    Space(String, u8),
    Org(u8),
}

#[derive(Debug, PartialEq)]
enum Instruction {
    Add(String),
    Lda(String),
    Sta(String),
    Hlt,
    Nop,
    Jmp(String),
    Jz(String),
    Jn(String),
    Not(String),
    Or(String),
    And(String),
}

pub struct Program {
    setup: Vec<DataDecl>,
    text: Vec<Instruction>,
}

pub struct ParserT<'a> {
    pub lexer: Lexer<'a>,
    pub lookahead: Option<Token<'a>>,
    pub valid: bool,
    pub symbols: SymbolTable,
    pub program: Option<Program>,
}

impl<'a> ParserT<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let next_t = lexer.next_token();

        let (first_token, is_valid) = match next_t {
            Some(Ok(token)) => (Some(token), true),
            Some(Err(e)) => {
                println!("Error: lexical error at init: {}", e);
                (None, false)
            }
            None => (None, true),
        };

        Self {
            lexer,
            lookahead: first_token,
            valid: is_valid,
            symbols: SymbolTable::new(),
            program: None,
        }
    }

    fn peek_kind(&self) -> Option<TokenType<'a>> {
        Some(self.lookahead.as_ref().unwrap().kind)
    }

    fn advance(&mut self) -> Result<(), LexerError> {
        match self.lexer.next_token() {
            Some(Ok(token)) => {
                self.lookahead = Some(token);
                Ok(())
            }
            Some(Err(e)) => {
                self.lookahead = None;
                self.valid = false;
                Err(e)
            }
            None => {
                self.lookahead = None;
                Ok(())
            }
        }
    }

    fn consume_blanks(&mut self) -> Result<(), LexerError> {
        while let Some(TokenType::NewLine) = self.peek_kind() {
            self.advance()?;
        }
        Ok(())
    }

    fn expect(&mut self, expected: TokenType<'a>) -> Result<(), LexerError> {
        match self.peek_kind() {
            Some(kind) if kind == expected => {
                self.advance()?;
                Ok(())
            }
            Some(t) => {
                let line = self.lookahead.as_ref().unwrap().line;
                let str_error = format!(
                    "Syntax error at {} expected '{}' but found '{}'",
                    line, expected, t
                );
                Err(LexerError::new(str_error))
            }
            None => Err(LexerError::new(format!(
                "Unexpected end of file. Expected {}",
                expected
            ))),
        }
    }

    fn expect_identifier(&mut self) -> Result<&'a str, LexerError> {
        match self.peek_kind() {
            Some(TokenType::Identfier(name)) => {
                let extrected_name = name;
                self.advance()?;
                Ok(extrected_name)
            }
            Some(kind) => {
                let line = self.lookahead.as_ref().unwrap().line;
                Err(LexerError::new(format!(
                    "Syntax error at {}, expected Identifier, found {}",
                    line, kind
                )))
            }
            None => Err(LexerError::new(
                "End of file, expected identifier".to_string(),
            )),
        }
    }

    fn expect_label(&mut self, expected_label: &str) -> Result<(), LexerError> {
        match self.peek_kind() {
            Some(TokenType::Label(name)) if name == expected_label => {
                self.advance()?;
                Ok(())
            }
            Some(kind) => {
                let line = self.lookahead.as_ref().unwrap().line;
                Err(LexerError::new(format!(
                    "Syntax error at {}, expected label '{}', but found {}",
                    line, expected_label, kind
                )))
            }
            None => {
                let line = self.lookahead.as_ref().unwrap().line;
                Err(LexerError::new(format!(
                    "Unexpected end of file. Expected label '{}' at line {}",
                    expected_label, line
                )))
            }
        }
    }

    fn expect_number(&mut self) -> Result<u8, LexerError> {
        match self.peek_kind() {
            Some(TokenType::Num(value)) => {
                let extracted_number = value;
                self.advance()?;
                Ok(extracted_number)
            }
            Some(kind) => {
                let line = self.lookahead.as_ref().unwrap().line;
                Err(LexerError::new(format!(
                    "Syntax error at line {}. Expected 'u8' number, found {}",
                    line, kind
                )))
            }
            None => Err(LexerError::new(
                "Unexpected end of file. Expected 'u8' number".into(),
            )),
        }
    }

    fn expect_instruction(&mut self) -> Result<&'a str, LexerError> {
        match self.peek_kind() {
            Some(TokenType::Instruction(name)) => {
                let extracted_name = name;
                self.advance()?;
                Ok(extracted_name)
            }
            Some(kind) => {
                let line = self.lookahead.as_ref().unwrap().line;
                Err(LexerError::new(format!(
                    "Syntax error at line {}. Expected instruction found {}",
                    line, kind
                )))
            }
            None => Err(LexerError::new(
                "Unexpected end of file. Expected Instruction".into(),
            )),
        }
    }

    fn expect_data_declaration(&mut self) -> Result<&'a str, LexerError> {
        match self.peek_kind() {
            Some(TokenType::DataDeclaration(name)) => {
                let extracted_name = name;
                self.advance()?;
                Ok(extracted_name)
            }
            Some(kind) => {
                let line = self.lookahead.as_ref().unwrap().line;
                Err(LexerError::new(format!(
                    "Syntax error at line {}. Expected data declaration, found '{}'.",
                    line, kind
                )))
            }
            None => Err(LexerError::new(
                "Unexpected end of file at data section".into(),
            )),
        }
    }

    fn parse_data_stmt(&mut self) -> Result<DataDecl, LexerError> {
        match self.peek_kind() {
            Some(TokenType::DataDeclaration("ORG")) => {
                self.advance()?;
                let num = self.expect_number()?;
                Ok(DataDecl::Org(num))
            }
            Some(TokenType::Identfier(_)) => {
                let id = self.expect_identifier()?;
                let instr = self.expect_data_declaration()?;
                match instr {
                    "DATA" => {
                        let num = self.expect_number()?;
                        Ok(DataDecl::Data(id.to_string(), num))
                    }
                    "SPACE" => {
                        let num = self.expect_number()?;
                        Ok(DataDecl::Space(id.to_string(), num))
                    }
                    _ => {
                        let line = self.lookahead.as_ref().map_or(0, |t| t.line);
                        Err(LexerError::new(format!(
                            "Error at line {}. Expected 'DATA' or 'SPACE' but found '{}'",
                            line, instr
                        )))
                    }
                }
            }
            Some(kind) => {
                let line = self.lookahead.as_ref().unwrap().line;
                Err(LexerError::new(format!(
                    "Unexpected token at line {}. Expected data statement, but found '{}'",
                    line, kind
                )))
            }
            None => Err(LexerError::new(
                "Unexpected end of file at line. Expected data statement".into(),
            )),
        }
    }

    fn parse_data(&mut self) -> Result<Vec<DataDecl>, LexerError> {
        let mut data_statements = Vec::<DataDecl>::new();
        self.expect_label("setup")?;
        self.expect(TokenType::NewLine)?;

        while let Some(kind) = self.peek_kind() {
            match kind {
                TokenType::Label("end") => break,
                TokenType::NewLine => {
                    self.advance()?;
                }
                _ => {
                    let data = self.parse_data_stmt()?;
                    data_statements.push(data);
                }
            }
        }

        self.expect_label("end")?;
        Ok(data_statements)
    }

    fn parse_instruction(&mut self) -> Result<Instruction, LexerError> {
        let instr = self.expect_instruction()?;
        match instr {
            "hlt" => Ok(Instruction::Hlt),
            "nop" => Ok(Instruction::Nop),
            _ => {
                let id = self.expect_identifier()?;
                match instr {
                    "add" => Ok(Instruction::Add(id.to_string())),
                    "lda" => Ok(Instruction::Lda(id.to_string())),
                    "sta" => Ok(Instruction::Sta(id.to_string())),
                    "jmp" => Ok(Instruction::Jmp(id.to_string())),
                    "jn" => Ok(Instruction::Jn(id.to_string())),
                    "jz" => Ok(Instruction::Jz(id.to_string())),
                    "or" => Ok(Instruction::Or(id.to_string())),
                    "not" => Ok(Instruction::Not(id.to_string())),
                    "and" => Ok(Instruction::And(id.to_string())),
                    _ => Err(LexerError::new(format!(
                        "Expected 'instruction' at line {}, but found token '{}'",
                        self.lookahead.as_ref().unwrap().line,
                        instr
                    ))),
                }
            }
        }
    }

    fn parse_text(&mut self) -> Result<Vec<Instruction>, LexerError> {
        let mut instructions = Vec::<Instruction>::new();
        self.expect_label("text")?;
        while let Some(kind) = self.peek_kind() {
            match kind {
                TokenType::Label("end") => break,
                TokenType::NewLine => self.advance()?,
                _ => {
                    let instr = self.parse_instruction()?;
                    instructions.push(instr);
                }
            }
        }
        self.expect_label("end")?;
        Ok(instructions)
    }

    fn parse_program(&mut self) -> Result<Program, LexerError> {
        self.consume_blanks()?;
        let data = self.parse_data()?;

        self.consume_blanks()?;
        let text = self.parse_text()?;

        let p = Program { setup: data, text };
        Ok(p)
    }

    fn generate_binary(&self) -> Result<Vec<u8>, LexerError> {
        let mut mem = vec![0u8; 516];
        let mut pc = self.symbols.program_counter as usize;
        let program = self.program.as_ref().expect("Progam was not parsed yet.");

        for (addr, value) in self.symbols.map.values() {
            mem[(*addr as usize) * 2 + 4] = *value;
        }

        for instruction in &program.text {
            match instruction {
                Instruction::Nop => {
                    mem[pc] = 0u8;
                    pc += 2;
                }
                Instruction::Hlt => {
                    mem[pc] = 240u8;
                    pc += 2;
                }
                Instruction::Add(var_name) => {
                    mem[pc] = 48;
                    pc += 2;

                    match self.symbols.map.get(var_name) {
                        Some(&item) => {
                            mem[pc] = item.0;
                            pc += 2;
                        }
                        None => {
                            return Err(LexerError::new(format!(
                                "Semantic error. Var '{}' was not defined.",
                                var_name
                            )));
                        }
                    }
                }
                Instruction::Sta(var_name) => {
                    mem[pc] = 16u8;
                    pc += 2;

                    match self.symbols.map.get(var_name) {
                        Some(&item) => {
                            mem[pc] = item.0;
                            pc += 2;
                        }
                        None => {
                            return Err(LexerError::new(format!(
                                "Semantic error. Var '{}' was not defined.",
                                var_name
                            )));
                        }
                    }
                }
                Instruction::Lda(var_name) => {
                    mem[pc] = 32u8;
                    pc += 2;

                    match self.symbols.map.get(var_name) {
                        Some(&item) => {
                            mem[pc] = item.0;
                            pc += 2;
                        }
                        None => {
                            return Err(LexerError::new(format!(
                                "Semantic error. Var '{}' was not defined.",
                                var_name
                            )));
                        }
                    }
                }
                Instruction::Or(var_name) => {
                    mem[pc] = 64u8;
                    pc += 2;

                    match self.symbols.map.get(var_name) {
                        Some(&item) => {
                            mem[pc] = item.0;
                            pc += 2;
                        }
                        None => {
                            return Err(LexerError::new(format!(
                                "Semantic error. Var '{}' was not defined.",
                                var_name
                            )));
                        }
                    }
                }
                Instruction::And(var_name) => {
                    mem[pc] = 80u8;
                    pc += 2;

                    match self.symbols.map.get(var_name) {
                        Some(&item) => {
                            mem[pc] = item.0;
                            pc += 2;
                        }
                        None => {
                            return Err(LexerError::new(format!(
                                "Semantic error. Var '{}' was not defined.",
                                var_name
                            )));
                        }
                    }
                }
                Instruction::Not(var_name) => {
                    mem[pc] = 96u8;
                    pc += 2;

                    match self.symbols.map.get(var_name) {
                        Some(&item) => {
                            mem[pc] = item.0;
                            pc += 2;
                        }
                        None => {
                            return Err(LexerError::new(format!(
                                "Semantic error. Var '{}' was not defined.",
                                var_name
                            )));
                        }
                    }
                }
                Instruction::Jmp(var_name) => {
                    mem[pc] = 128u8;
                    pc += 2;

                    match self.symbols.map.get(var_name) {
                        Some(&item) => {
                            mem[pc] = item.0;
                            pc += 2;
                        }
                        None => {
                            return Err(LexerError::new(format!(
                                "Semantic error. Var '{}' was not defined.",
                                var_name
                            )));
                        }
                    }
                }
                Instruction::Jn(var_name) => {
                    mem[pc] = 144u8;
                    pc += 2;

                    match self.symbols.map.get(var_name) {
                        Some(&item) => {
                            mem[pc] = item.0;
                            pc += 2;
                        }
                        None => {
                            return Err(LexerError::new(format!(
                                "Semantic error. Var '{}' was not defined.",
                                var_name
                            )));
                        }
                    }
                }
                Instruction::Jz(var_name) => {
                    mem[pc] = 160u8;
                    pc += 2;

                    match self.symbols.map.get(var_name) {
                        Some(&item) => {
                            mem[pc] = item.0;
                            pc += 2;
                        }
                        None => {
                            return Err(LexerError::new(format!(
                                "Semantic error. Var '{}' was not defined.",
                                var_name
                            )));
                        }
                    }
                }
            }
        }

        Ok(mem)
    }

    pub fn parse(&mut self) -> Result<(), LexerError> {
        let parsed_program = self.parse_program()?;
        self.symbols.build(&parsed_program.setup)?;
        self.program = Some(parsed_program);
        self.generate_binary()?;
        Ok(())
    }
}

pub type Item = (u8, u8);

#[derive(Debug)]
pub struct SymbolTable {
    pub program_counter: u8,
    pub map: HashMap<String, Item>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            program_counter: 4,
            map: HashMap::new(),
        }
    }

    fn build(&mut self, declarations: &[DataDecl]) -> Result<(), LexerError> {
        let mut current_addr: u8 = 250;

        for decl in declarations {
            match decl {
                DataDecl::Data(var, value) => {
                    if self.map.contains_key(var) {
                        return Err(LexerError::new(format!(
                            "Semantic error. Var '{}' was already declared.",
                            var
                        )));
                    }

                    self.map.insert(var.clone(), (current_addr, *value));
                    current_addr -= 1;
                }
                DataDecl::Space(var, size) => {
                    if self.map.contains_key(var) {
                        return Err(LexerError::new(format!(
                            "Semantic error. Var '{}' was already declared.",
                            var
                        )));
                    }

                    current_addr -= size - 1;

                    self.map.insert(var.clone(), (current_addr, 0u8));
                }
                DataDecl::Org(addr) => {
                    if self.program_counter != 4 || *addr < 4u8 {
                        return Err(LexerError::new(
                            "Semantic error. Program count was already set.".into(),
                        ));
                    }
                    self.program_counter = *addr;
                }
            }
        }
        Ok(())
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
