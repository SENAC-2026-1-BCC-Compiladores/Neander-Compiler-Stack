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
pub enum TokenType<'a> {
    // Delimitadores Iniciadores
    Label(&'a str),

    // Variaveis
    Identfier(&'a str),
    Variable(&'a str),

    // Instrucoes
    Instruction(&'a str),

    // Literais
    Num(i8),

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

struct NodeT<'a> {
    pub left: Option<Box<NodeT<'a>>>,
    pub right: Option<Box<NodeT<'a>>>,
    pub kind: TokenType<'a>,
}

struct ParserT<'a> {
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
    pub error: Option<LexerError>,
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
            error: None,
            line: 1,
            col: 0,
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

    fn get_reserved_token(lexeme: &'a str) -> TokenType<'a> {
        match lexeme {
            "data" | "text" | "end" => TokenType::Label(lexeme),
            "nop" | "add" | "sta" | "lda" | "or" |
            "and" | "not" | "jmp" | "jn" | "jz" | "hlt"
                => TokenType::Instruction(lexeme),
            _ => TokenType::Identfier(lexeme),
        }
    } 

    fn skip_blank(&mut self) {
        while let Some(c) = self.peek()  {
            if c == '\n' || c == '\t' {
                self.consume();
            } else {
                break;
            }
        }        
    } 

    fn next_token (&mut self) -> Option<Result<Token, LexerError>> {
        let c = self.consume()?;
        self.skip_blank();
        match c {
            _ => {
                let error_str = format!("Error: unexpected symbol at line: {}");
                return Some(Err(LexerError::new(error_str)));
            }
        };
        while let Some(c) = self.consume() {
            if self.error.is_some() {
                break;
            }

            match c {
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
                            return Token::new(
                                TokenType::Variable(lexeme),
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
                        .push(Token::new(TokenType::Colon, lexeme, self.line));
                }
                '!' => {
                    let lexeme = &self.stream[self.position..self.position];
                    self.tokens
                        .push(Token::new(TokenType::Bang, lexeme, self.line));
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
                            self.tokens
                                .push(Token::new(TokenType::Arrow, lexeme, self.line));
                        }
                    } else {
                        let lexeme = &self.stream[start_pos..self.position];
                        self.tokens
                            .push(Token::new(TokenType::Minus, lexeme, self.line));
                    }
                }
                '=' => {
                    let lexeme = &self.stream[self.position..self.position];
                    self.tokens
                        .push(Token::new(TokenType::Equals, lexeme, self.line));
                }
                ',' => {
                    let lexeme = &self.stream[self.position..self.position];
                    self.tokens
                        .push(Token::new(TokenType::Comma, lexeme, self.line));
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
                        self.tokens.push(Token::new(kind, lexeme, self.line));
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
                        let num: i8 = match lexeme.parse() {
                            Ok(n) => n,
                            Err(_) => {
                                let error_str = format!("Invalid u8 at line {}", self.line);
                                let error = LexerError::new(error_str);
                                self.error = Some(error);
                                0
                            }
                        };
                        self.tokens
                            .push(Token::new(TokenType::Num(num), lexeme, self.line));
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

impl<'a> ParserT<'a> {
    fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lookahead: Some(lexer.tokens[0]),
            valid: true,
        }
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
