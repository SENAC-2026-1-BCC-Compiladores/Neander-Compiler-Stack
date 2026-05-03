use std::fmt;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TokenType {
    Number(u8),
    Plus,
}

#[derive(PartialEq, Debug)]
pub struct Token {
    pub kind: TokenType,
    pub pos: usize,
}

impl Token {
    pub fn new(kind: TokenType, pos: usize) -> Self {
        Token { kind, pos }
    }
}

#[derive(Debug)]
pub struct LexerError {
    pub error: String,
}

impl LexerError {
    pub fn new(message: String) -> Self {
        LexerError { error: message }
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lexical Error: {}", self.error)
    }
}

impl std::error::Error for LexerError {}

pub struct Lexer<'a> {
    pub stream: &'a str,
    pub pos: usize,
    pub col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(stream: &'a str) -> Self {
        Lexer {
            stream,
            pos: 0,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.stream[self.pos..].chars().next()
    }

    fn consume(&mut self) -> Option<char> {
        if let Some(c) = self.peek() {
            self.pos += c.len_utf8();
            self.col += 1;
            Some(c)
        } else {
            None
        }
    }

    fn skip_blank(&mut self) {
        while let Some(' ' | '\t') = self.peek() {
            self.consume();
        }
    }

    fn consume_numeric(&mut self) -> Result<Token, LexerError> {
        // consume first char
        let start_idx = self.pos;
        let position = self.col;
        self.consume();

        while let Some(n) = self.peek() {
            if n.is_ascii_digit() {
                self.consume();
            } else {
                break;
            }
        }

        let lexeme = &self.stream[start_idx..self.pos];
        match lexeme.parse::<u8>() {
            Ok(num) => Ok(Token::new(TokenType::Number(num), position)),
            Err(_) => Err(LexerError::new(format!(
                "Number '{}' out of bounds at {}. Expected u8 number (0-255).",
                lexeme, position
            ))),
        }
    }

    pub fn next_token(&mut self) -> Option<Result<Token, LexerError>> {
        self.skip_blank();
        let c = self.peek()?;
        match c {
            c if c.is_ascii_digit() => Some(self.consume_numeric()),
            '+' => {
                self.consume();
                Some(Ok(Token::new(TokenType::Plus, self.col)))
            }
            _ => Some(Err(LexerError::new(format!(
                "Unexpected token at {}.",
                self.col
            )))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let mut lexer = Lexer::new("213 + 13");
        let t = lexer.next_token().unwrap().unwrap();

        assert_eq!(t.kind, TokenType::Number(213));

        let t = lexer.next_token().unwrap().unwrap();
        assert_eq!(t.kind, TokenType::Plus);

        let t = lexer.next_token().unwrap().unwrap();
        assert_eq!(t.kind, TokenType::Number(13));
    }
}
