use clap::Parser;
use std::error::Error;
use std::io::Read;
use std::{fmt, fs, io};

#[derive(Parser)]
struct Cli {
    #[arg(long, short)]
    path: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum TokenType {
    // Delimitadores Iniciadores
    Label,

    // Variaveis
    Identfier,
    Variable,

    // Instrucoes
    Instruction,

    // Literais
    Num,

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

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            TokenType::Label => "Label",
            TokenType::Identfier => "Identfier",
            TokenType::Variable => "Variable",
            TokenType::Semicolon => "Semicolon",
            TokenType::Instruction => "Instruction",
            TokenType::Minus => "Minus",
            TokenType::Num => "Number",
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

pub struct Token<'a> {
    kind: TokenType,
    lexeme: &'a str,
    literal: Option<&'a str>,
    line: usize,
}

impl<'a> Token<'a> {
    fn new(kind: TokenType, lexeme: &'a str, literal: &'a str, line: usize) -> Self {
        Token {
            kind,
            lexeme,
            literal: Some(literal),
            line,
        }
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
    pub ch: char,
    pub error: Option<LexerError>,
    pub line: usize,
}

impl<'a> Lexer<'a> {
    fn new(stream: &'a str) -> Self {
        Lexer {
            stream,
            tokens: vec![],
            position: 0,
            ch: '\0',
            error: None,
            line: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.stream[self.position..].chars().next()
    }

    fn consume(&mut self) -> Option<char> {
        if let Some(c) = self.peek() {
            self.position += c.len_utf8();
            Some(c)
        } else {
            None
        }
    }

    fn get_reserved_token(lexeme: &str) -> TokenType {
        match lexeme {
            "data" | "text" | "end" => TokenType::Label,
            "add" | "sub" | "mul" | "div" => TokenType::Instruction,
            _ => TokenType::Identfier,
        }
    }

    fn run(&mut self) {
        // enquanto funcao avancar funcionar continua
        while let Some(c) = self.consume() {
            if self.error.is_some() {
                break;
            }

            match c {
                // ignorar espaços
                // pular comentários
                ' ' | '\t' | '\r' => {
                    continue;
                }
                '\n' => {
                    self.line += 1;
                }
                '@' => {
                    let start_pos = self.position - 1;

                    if let Some(next_char) = self.peek() {
                        if next_char.is_alphabetic() {
                            self.consume();

                            while let Some(c) = self.peek() {
                                if c.is_alphabetic() {
                                    self.consume();
                                } else {
                                    break;
                                }
                            }

                            let lexeme = &self.stream[start_pos..self.position];
                            self.tokens.push(Token::new(
                                TokenType::Variable,
                                lexeme,
                                lexeme,
                                self.line,
                            ));
                        }
                    } else {
                        let error_str =
                            format!("Error: invalid format after {} at line {}", c, self.line);
                        let error = LexerError::new(error_str);
                        self.error = Some(error);
                    }
                }
                ':' => {
                    let lexeme = &self.stream[self.position..self.position];
                    self.tokens
                        .push(Token::new(TokenType::Colon, lexeme, lexeme, self.line));
                }
                '!' => {
                    let lexeme = &self.stream[self.position..self.position];
                    self.tokens
                        .push(Token::new(TokenType::Bang, lexeme, lexeme, self.line));
                }
                ';' => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.consume();
                    }
                }
                '-' => {
                    let start_pos = self.position;
                    if let Some(next_char) = self.peek() {
                        if next_char == '>' {
                            self.consume();

                            let lexeme = &self.stream[start_pos..self.position];
                            self.tokens.push(Token::new(
                                TokenType::Arrow,
                                lexeme,
                                lexeme,
                                self.line,
                            ));
                        }
                    } else {
                        let lexeme = &self.stream[start_pos..self.position];
                        self.tokens
                            .push(Token::new(TokenType::Minus, lexeme, lexeme, self.line));
                    }
                }
                '=' => {
                    let lexeme = &self.stream[self.position..self.position];
                    self.tokens
                        .push(Token::new(TokenType::Equals, lexeme, lexeme, self.line));
                }
                ',' => {
                    let lexeme = &self.stream[self.position..self.position];
                    self.tokens
                        .push(Token::new(TokenType::Comma, lexeme, lexeme, self.line));
                }
                _ => {
                    if c.is_alphabetic() {
                        let start_pos = self.position - 1;

                        while let Some(ch) = self.peek() {
                            if ch.is_alphanumeric() || ch == '_' {
                                self.consume();
                            } else {
                                break;
                            }
                        }
                        let lexeme = &self.stream[start_pos..self.position];
                        let kind = Lexer::get_reserved_token(lexeme);
                        self.tokens
                            .push(Token::new(kind, lexeme, lexeme, self.line));
                    } else if c.is_numeric() {
                        let start_pos = self.position - 1;

                        while let Some(ch) = self.peek() {
                            if ch.is_numeric() {
                                self.consume();
                            } else {
                                break;
                            }
                        }

                        let lexeme = &self.stream[start_pos..self.position];
                        self.tokens
                            .push(Token::new(TokenType::Num, lexeme, lexeme, self.line));
                    } else {
                        let error_str =
                            format!("Unexpected symbol \"{}\" at line {}", c, self.line);
                        let error = LexerError::new(error_str);
                        self.error = Some(error);
                    }
                }
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

    let mut lexer = Lexer::new(&data);
    lexer.run();

    for t in lexer.tokens {
        println!("{}", t);
    }

    if let Some(e) = lexer.error {
        println!("{}", e);
    }

    Ok(())
}
