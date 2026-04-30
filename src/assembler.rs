use clap::Parser;
use std::collections::btree_map::ExtractIf;
use std::error::Error;
use std::io::Read;
use std::{fmt, fs, io};

#[derive(Parser)]
struct Cli {
    #[arg(long, short)]
    path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType<'a> {
    // Delimitadores Iniciadores
    Label(&'a str),

    // Variaveis
    Identfier(&'a str),
    Variable(&'a str),

    // Instrucoes
    Instruction(&'a str),

    // Literais
    Num(u8),

    // Simbolos unicos
    Equals,
    Bang,
    Colon,
    Semicolon,
    Arrow,
    Comma,
    NewLine,
    Minus,
}

#[derive(Debug, PartialEq)]
enum DataDecl {
    Data(String, u8),
    Space(String, u8),
    Org(u8),
}

enum Instruction {
    Add(String),
    Lda(String),
    Sta(String),
    Hlt,
    Nop,
    Jmp(u8),
    Jz(u8),
    Jn(u8),
    Not(String),
    Or(String),
    And(String),
}

struct Program {
    setup: Vec<DataDecl>,
    text: Vec<Instruction>,
}

struct ParserT<'a> {
    pub lexer: Lexer<'a>,
    pub lookahead: Option<Token<'a>>,
    pub valid: bool,
}

impl<'a> fmt::Display for TokenType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            TokenType::Label(_) => "Label",
            TokenType::Identfier(_) => "Identfier",
            TokenType::Variable(_) => "Variable",
            TokenType::Semicolon => "Semicolon",
            TokenType::Instruction(_) => "Instruction",
            TokenType::Minus => "Minus",
            TokenType::Num(_) => "Number",
            TokenType::Equals => "Equals",
            TokenType::Bang => "Bang",
            TokenType::Colon => "Colon",
            TokenType::Arrow => "Arrow",
            TokenType::Comma => "Comma",
            TokenType::NewLine => "New Line",
        };

        write!(f, "{}", name)
    }
}

#[derive(PartialEq)]
pub struct Token<'a> {
    kind: TokenType<'a>,
    lexeme: &'a str,
    line: usize,
}

impl<'a> Token<'a> {
    fn new(kind: TokenType<'a>, lexeme: &'a str, line: usize) -> Self {
        Token { kind, lexeme, line }
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Token -> \n\tTipo: {} \n\tLexeme: {} \n\tLine: {}",
            self.kind,
            self.lexeme.escape_debug(),
            self.line
        )
    }
}

pub struct Lexer<'a> {
    stream: &'a str,
    pub tokens: Vec<Token<'a>>,
    pub position: usize,
    pub cursor: usize,
    pub error: bool,
    pub line: usize,
    pub col: usize,
}

impl<'a> Lexer<'a> {
    fn new(stream: &'a str) -> Self {
        Lexer {
            stream,
            tokens: vec![],
            position: 0,
            cursor: 0,
            error: false,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.stream[self.position..].chars().next()
    }

    fn consume(&mut self) -> Option<char> {
        if let Some(c) = self.peek() {
            self.position += c.len_utf8();
            self.col += 1;
            Some(c)
        } else {
            None
        }
    }

    fn get_reserved_token(lexeme: &'a str) -> TokenType<'a> {
        match lexeme {
            "setup" | "text" | "end" => TokenType::Label(lexeme),
            "nop" | "DATA" | "SPACE" | "ORG" | "add" | "sta" | "lda" | "or" | "and" | "not"
            | "jmp" | "jn" | "jz" | "hlt" => TokenType::Instruction(lexeme),
            _ => TokenType::Identfier(lexeme),
        }
    }

    fn skip_blank(&mut self) {
        while let Some(' ' | '\t') = self.peek() {
            self.consume();
        }
    }

    fn new_line(&mut self) -> Result<Token<'a>, LexerError> {
        // consuming the first '\n'
        self.consume();
        self.line += 1;
        self.col = 1;

        // check for others \n
        while let Some(c) = self.peek() {
            if c != '\n' {
                break;
            }
            self.consume();
            self.line += 1;
            self.col = 1;
        }

        Ok(Token::new(TokenType::NewLine, "\n", self.line))
    }

    fn ignore_comments(&mut self) {
        if let Some(';') = self.peek() {
            self.consume();

            while let Some(stop) = self.peek() {
                if stop == '\n' {
                    break;
                } else {
                    self.consume();
                }
            }
        }
    }

    fn consume_alpha(&mut self) -> Result<Token<'a>, LexerError> {
        let start_idx = self.position;

        while let Some(c) = self.peek() {
            if !c.is_alphabetic() {
                break;
            } else {
                self.consume();
            }
        }

        let kind = Lexer::get_reserved_token(&self.stream[start_idx..self.position]);
        Ok(Token::new(
            kind,
            &self.stream[start_idx..self.position],
            self.line,
        ))
    }

    fn consume_numeric(&mut self) -> Result<Token<'a>, LexerError> {
        let start_idx = self.position;

        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() {
                break;
            } else {
                self.consume();
            }
        }

        let lex_str = &self.stream[start_idx..self.position];
        match lex_str.parse::<u8>() {
            Ok(num) => Ok(Token::new(TokenType::Num(num), lex_str, self.line)),
            Err(_) => {
                let str_error = format!(
                    "Error: number '{}' out of bounds expected (0-255) at: {}:{}",
                    lex_str, self.line, self.col
                );
                Err(LexerError::new(str_error))
            }
        }
    }

    fn next_token(&mut self) -> Option<Result<Token<'a>, LexerError>> {
        self.skip_blank();
        self.ignore_comments();
        let c = self.peek()?;
        match c {
            '\n' => Some(self.new_line()),
            c if c.is_alphabetic() => Some(self.consume_alpha()),
            c if c.is_numeric() => Some(self.consume_numeric()),
            _ => {
                let error_str = format!(
                    "Error: unexpected symbol '{}' at {}:{}",
                    c, self.line, self.col
                );
                Some(Err(LexerError::new(error_str)))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LexerError {
    message: String,
}

impl LexerError {
    fn new(error: String) -> Self {
        LexerError { message: error }
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ERROR: {}", self.message)
    }
}

impl<'a> ParserT<'a> {
    fn new(mut lexer: Lexer<'a>) -> Self {
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
        }
    }

    fn peek(&self) -> Option<&Token<'a>> {
        self.lookahead.as_ref()
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

    fn parse_data_stmt(&mut self) -> Result<DataDecl, LexerError> {
        match self.peek_kind() {
            Some(TokenType::Instruction("ORG")) => {
                self.advance()?;
                let num = self.expect_number()?;
                Ok(DataDecl::Org(num))
            }
            Some(TokenType::Identfier(_)) => {
                let id = self.expect_identifier()?;
                let instr = self.expect_instruction()?;
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

    fn parse_program(&mut self) -> Result<(), LexerError> {
        Ok(())
    }

    fn parse(&mut self) -> Result<(), LexerError> {
        Ok(())
    }
}

impl Error for LexerError {}

pub fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let data: String;
    let mut buff = String::new();

    // path para arquivo .asm
    if let Some(path) = cli.path {
        data = fs::read_to_string(path)?;
    } else {
        io::stdin().read_to_string(&mut buff)?;
        if buff.trim().is_empty() {
            return Err("Erro: Arquivo fornecido vazio ou inexistente".into());
        }

        data = buff;
    }

    let lexer = Lexer::new(&data);
    let mut parser = ParserT::new(lexer);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_data_section() {
        let source = "setup\n\n\n\n A DATA 44\n\n\n\n\n\n ORG 4\n\n B SPACE 5\n\n end";
        let lexer = Lexer::new(source);
        let mut parser = ParserT::new(lexer);

        let result = parser.parse_data();

        assert!(
            result.is_ok(),
            "Error returned from parser: {:?}",
            result.err()
        );

        let statements = result.unwrap();
        assert_eq!(statements.len(), 3);

        assert_eq!(statements[0], DataDecl::Data("A".to_string(), 44));
        assert_eq!(statements[1], DataDecl::Org(4));
        assert_eq!(statements[2], DataDecl::Space("B".to_string(), 5));
    }
}
