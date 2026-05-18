use crate::assembler::codegen::Codegen;
use crate::assembler::*;

#[derive(Debug, PartialEq)]
pub enum DataDecl {
    Data(String, u8),
    Space(String, u8),
    Org(u8),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Register {
    T0,
    T1,
    T2,
    T3,
    T4,
}

impl Register {
    pub fn address(&self) -> u8 {
        match self {
            Register::T0 => 251,
            Register::T1 => 252,
            Register::T2 => 253,
            Register::T3 => 254,
            Register::T4 => 255,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operand {
    Register(Register),
    Symbol(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Add(Operand),
    Sub(Operand),
    Lda(Operand),
    Sta(Operand),
    Hlt,
    Nop,
    Jmp(Operand),
    Jz(Operand),
    Jn(Operand),
    Not,
    Or(Operand),
    And(Operand),
}

pub struct Program {
    pub setup: Vec<DataDecl>,
    pub text: Vec<Instruction>,
}

pub struct ParserT<'a> {
    pub lexer: Lexer<'a>,
    pub lookahead: Option<Token<'a>>,
    pub valid: bool,
    pub symbols: Codegen,
    pub program: Option<Program>,
}

macro_rules! expect_token {
    ($self:ident, $pattern:pat => $extracted:expr, $err_expected:expr) => {
        match $self.peek_kind() {
            Some($pattern) => {
                let res = $extracted;
                $self.advance()?;
                Ok(res)
            }
            Some(wrong_kind) => {
                let line = $self.current_line();
                Err(LexerError::new(format!(
                    "Syntax error at line {}. Expected {}, but found '{}'.",
                    line, $err_expected, wrong_kind
                )))
            }
            None => Err(LexerError::new(format!(
                "Unexpected end of file. Expecting {}",
                $err_expected
            ))),
        }
    };
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
            symbols: Codegen::new(),
            program: None,
        }
    }

    fn peek_kind(&self) -> Option<TokenType<'a>> {
        self.lookahead.as_ref().map(|t| t.kind)
    }

    fn current_line(&self) -> usize {
        self.lookahead.as_ref().map_or(self.lexer.line, |t| t.line)
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

    fn expect_operand(&mut self) -> Result<Operand, LexerError> {
        match self.peek_kind() {
            Some(TokenType::Register(name)) => {
                let op = match name {
                    "t0" => Operand::Register(Register::T0),
                    "t1" => Operand::Register(Register::T1),
                    "t2" => Operand::Register(Register::T2),
                    "t3" => Operand::Register(Register::T3),
                    "t4" => Operand::Register(Register::T4),
                    _ => return Err(LexerError::new(format!("Unkown register '{}'", name))),
                };

                self.advance()?;
                Ok(op)
            }

            Some(TokenType::Identfier(name)) => {
                let op = Operand::Symbol(name.to_string());
                self.advance()?;
                Ok(op)
            }

            Some(wrong_kind) => Err(LexerError::new(format!(
                "Expected Operand at line {} but found '{}'",
                self.current_line(),
                wrong_kind
            ))),

            None => Err(LexerError::new("Unexpected EOF".to_string())),
        }
    }

    fn expect_identifier(&mut self) -> Result<&'a str, LexerError> {
        expect_token!(self, TokenType::Identfier(name) => name, "Identifier")
    }

    fn expect_number(&mut self) -> Result<u8, LexerError> {
        expect_token!(self, TokenType::Num(num) => num, "u8 number (0-255)")
    }

    fn expect_instruction(&mut self) -> Result<&'a str, LexerError> {
        expect_token!(self, TokenType::Instruction(instr) => instr, "instruction")
    }

    fn expect_data_declaration(&mut self) -> Result<&'a str, LexerError> {
        expect_token!(self, TokenType::DataDeclaration(decl) => decl, "data statement")
    }

    fn expect_label(&mut self, expected_label: &str) -> Result<(), LexerError> {
        match self.peek_kind() {
            Some(TokenType::Label(name)) if name == expected_label => {
                self.advance()?;
                Ok(())
            }
            Some(kind) => {
                let line = self.current_line();
                Err(LexerError::new(format!(
                    "Syntax error at {}, expected label '{}', but found {}",
                    line, expected_label, kind
                )))
            }
            None => {
                let line = self.current_line();
                Err(LexerError::new(format!(
                    "Unexpected end of file. Expected label '{}' at line {}",
                    expected_label, line
                )))
            }
        }
    }

    fn parse_section<T, F>(
        &mut self,
        secntion_name: &str,
        mut parse_item: F,
    ) -> Result<Vec<T>, LexerError>
    where
        F: FnMut(&mut Self) -> Result<T, LexerError>,
    {
        let mut items = Vec::new();
        self.expect_label(secntion_name)?;
        self.consume_blanks()?;

        while let Some(kind) = self.peek_kind() {
            match kind {
                TokenType::Label("end") => break,
                TokenType::NewLine => self.advance()?,
                _ => {
                    items.push(parse_item(self)?);
                }
            }
        }

        self.expect_label("end")?;
        Ok(items)
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
                        let line = self.current_line();
                        Err(LexerError::new(format!(
                            "Error at line {}. Expected 'DATA' or 'SPACE' but found '{}'",
                            line, instr
                        )))
                    }
                }
            }
            Some(kind) => {
                let line = self.current_line();
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
        self.parse_section("setup", |parser| parser.parse_data_stmt())
    }

    fn parse_instruction(&mut self) -> Result<Instruction, LexerError> {
        let instr = self.expect_instruction()?;

        match instr {
            "hlt" => Ok(Instruction::Hlt),
            "nop" => Ok(Instruction::Nop),
            "not" => Ok(Instruction::Not),
            _ => {
                let op = self.expect_operand()?;
                match instr {
                    "add" => Ok(Instruction::Add(op)),
                    "sub" => Ok(Instruction::Sub(op)),
                    "lda" => Ok(Instruction::Lda(op)),
                    "sta" => Ok(Instruction::Sta(op)),
                    "jmp" => Ok(Instruction::Jmp(op)),
                    "jn" => Ok(Instruction::Jn(op)),
                    "jz" => Ok(Instruction::Jz(op)),
                    "or" => Ok(Instruction::Or(op)),
                    "and" => Ok(Instruction::And(op)),
                    _ => Err(LexerError::new(format!(
                        "Expected 'instruction' at line {}, but found token '{}'",
                        self.current_line(),
                        instr
                    ))),
                }
            }
        }
    }

    fn parse_text(&mut self) -> Result<Vec<Instruction>, LexerError> {
        self.parse_section("text", |parser| parser.parse_instruction())
    }

    pub fn parse(&mut self) -> Result<Program, LexerError> {
        self.consume_blanks()?;
        let data = self.parse_data()?;

        self.consume_blanks()?;
        let text = self.parse_text()?;

        let p = Program { setup: data, text };
        Ok(p)
    }
}
