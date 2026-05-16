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

enum AST {
    Add(Box<AST>, Box<AST>),
    Number(u8),
}

pub struct CalcParser<'a> {
    lexer: Lexer<'a>,
    lookahead: Option<Token>,
}

impl<'a> CalcParser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let t = lexer.next_token();

        let first_token = match t {
            Some(Ok(token)) => token,
            Some(Err(e)) => {
                panic!("{}", e);
            }
            None => {
                panic!("Error: there was not a first token.");
            }
        };

        CalcParser {
            lexer,
            lookahead: Some(first_token),
        }
    }

    fn peek_kind(&self) -> Option<TokenType> {
        Some(self.lookahead.as_ref().unwrap().kind)
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
