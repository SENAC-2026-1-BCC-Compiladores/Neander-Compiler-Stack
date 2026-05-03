use crate::calc::lexer::{Lexer, LexerError, Token, TokenType};

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
}
