use crate::ast::Expr;
use crate::errors::KeyScriptError;
use crate::scanner::{Token, TokenType, Value};
pub struct Parser<'a> {
    pub tokens: Vec<Token>,
    current: usize,
    filename: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, filename: &'a str) -> Parser {
        Parser {
            tokens,
            current: 0,
            filename,
        }
    }

    pub fn parse(&mut self) -> Expr {
        self.primary()
    }
    fn primary(&mut self) -> Expr {
        if self.match_tokens(&[TokenType::Bool]) {
            return if self.previous().literal == Some(Value::Bool(true)) {
                Expr::Literal(Value::Bool(true))
            } else {
                Expr::Literal(Value::Bool(false))
            }
        }
        if self.match_tokens(&[TokenType::String]) {
            return Expr::Literal(Value::String(match self.previous().literal
            {
                Some(Value::String(s)) => s,
                _ => panic!("kys"),
            }));
        }
        if self.match_tokens(&[TokenType::Int, TokenType::Float]) {
            match self.previous().literal {
                Some(Value::Int(n)) => return Expr::Literal(Value::Int(n)),
                Some(Value::Float(n)) => return Expr::Literal(Value::Float(n)),
                _ => panic!("kys"),
            }
        }
        if self.match_tokens(&[TokenType::Identifier]) {
            return Expr::Variable(self.previous());
        }
        panic!("kys");
    }
    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        for tt in types {
            if self.check(&tt) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, t_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.peek().tt == t_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    fn is_at_end(&self) -> bool {
        self.peek().tt == TokenType::Eof
    }
    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }
    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
    fn error(&self, msg: &str) {
        KeyScriptError::error(
            KeyScriptError::ParserError,
            Some(msg),
            Some(self.peek().line),
            Some(self.filename));
    }
    fn consume(&mut self, t_type: TokenType, msg: &str) -> Token {
        if self.peek().tt == t_type {
            return self.advance();
        }
        self.error(msg);
        panic!("kys")
    }
}