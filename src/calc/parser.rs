use std::error::Error;
use std::fmt;

use crate::calc::lexer::{Lexer, LexerError, Token, TokenType};

#[derive(Debug)]
pub struct ParserError {
    pub error: String,
}

impl ParserError {
    pub fn new(message: String) -> Self {
        ParserError { error: message }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parser Error: {}", self.error)
    }
}

impl Error for ParserError {}

pub enum AST {
    Number(u8),
    Add(Box<AST>, Box<AST>),
    Sub(Box<AST>, Box<AST>),
}

impl AST {
    pub fn print(&self, level: usize) {
        let ident = "    ".repeat(level);

        match self {
            AST::Add(left, right) => {
                left.print(level + 1);
                std::println!("{}+", ident);
                right.print(level + 1);
            }
            AST::Sub(left, right) => {
                left.print(level + 1);
                std::println!("{}-", ident);
                right.print(level + 1);
            }
            AST::Number(num) => {
                std::println!("{}{}", ident, num);
            }
        }
    }
}

pub struct CalcParser<'a> {
    lexer: Lexer<'a>,
    lookahead: Option<Token>,
}

impl<'a> CalcParser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Result<Self, LexerError> {
        let lookahead = lexer.next_token().transpose()?;
        Ok(CalcParser { lexer, lookahead })
    }

    fn peek_kind(&self) -> Option<TokenType> {
        self.lookahead.as_ref().map(|t| t.kind)
    }

    fn advance(&mut self) -> Result<(), LexerError> {
        match self.lexer.next_token() {
            Some(Ok(t)) => {
                self.lookahead = Some(t);
                Ok(())
            }
            Some(Err(e)) => {
                self.lookahead = None;
                Err(e)
            }
            None => {
                self.lookahead = None;
                Ok(())
            }
        }
    }

    pub fn parse_expr(&mut self) -> Result<AST, Box<dyn Error>> {
        let mut left = self.parse_factor()?;

        while let Some(kind) = self.peek_kind() {
            if kind == TokenType::Plus || kind == TokenType::Minus {
                self.advance()?;
                let right = self.parse_factor()?;

                if kind == TokenType::Plus {
                    left = AST::Add(Box::new(left), Box::new(right));
                } else {
                    left = AST::Sub(Box::new(left), Box::new(right));
                }
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<AST, Box<dyn Error>> {
        match self.peek_kind() {
            Some(TokenType::Number(num)) => {
                let node = AST::Number(num);
                self.advance()?;
                Ok(node)
            }
            Some(kind) => Err(ParserError::new(format!(
                "Unexpected token: {:?}. Was expecting a number",
                kind
            ))
            .into()),
            None => Err(ParserError::new("Was expecting number".to_string()).into()),
        }
    }
}
