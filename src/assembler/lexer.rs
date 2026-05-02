use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType<'a> {
    // Delimitadores Iniciadores
    Label(&'a str),

    // Variaveis
    Identfier(&'a str),

    // Instrucoes
    Instruction(&'a str),
    DataDeclaration(&'a str),

    // Literais
    Num(u8),

    // Simbolos unicos
    Semicolon,
    NewLine,
}

impl<'a> fmt::Display for TokenType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            TokenType::Label(_) => "Label",
            TokenType::Identfier(_) => "Identfier",
            TokenType::Semicolon => "Semicolon",
            TokenType::DataDeclaration(_) => "Data Declaration",
            TokenType::Instruction(_) => "Instruction",
            TokenType::Num(_) => "Number",
            TokenType::NewLine => "New Line",
        };

        write!(f, "{}", name)
    }
}

#[derive(PartialEq)]
pub struct Token<'a> {
    pub kind: TokenType<'a>,
    pub lexeme: &'a str,
    pub line: usize,
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
    pub fn new(stream: &'a str) -> Self {
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
            "DATA" | "SPACE" | "ORG" => TokenType::DataDeclaration(lexeme),
            "nop" | "add" | "sta" | "lda" | "or" | "and" | "not" | "jmp" | "jn" | "jz" | "hlt" => {
                TokenType::Instruction(lexeme)
            }
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

    pub fn next_token(&mut self) -> Option<Result<Token<'a>, LexerError>> {
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
    pub fn new(error: String) -> Self {
        LexerError { message: error }
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ERROR: {}", self.message)
    }
}

impl Error for LexerError {}
