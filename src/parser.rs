use crate::ast::{Expr, Stmt};
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

    pub fn parse(&mut self) -> Stmt {
        self.expr_stmt()
    }

    fn expr_stmt(&mut self) -> Stmt {
        let stmt = Stmt::Expression(self.assignment());
        self.consume(TokenType::Semicolon, "missing ; at the end of the line");
        stmt
    }

    pub fn assignment(&mut self) -> Expr {
        let identifier = self.logical();
        if self.match_tokens(&[TokenType::Equal]) {
            let value = self.logical();
            match identifier {
                Expr::Variable(name) => {
                    return Expr::Assign {
                        name,
                        value: Box::new(value),
                    }
                }
                _ => {
                    self.error("cannot assign to a non variable");
                    panic!();
                }
            }
        }
        identifier
    }

    pub fn logical(&mut self) -> Expr {
        let mut left: Expr = self.equality();
        while self.match_tokens(&[TokenType::And, TokenType::Or]) {
            let operator = self.previous().clone();
            let right: Expr = self.equality();
            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        left
    }

    fn equality(&mut self) -> Expr {
        let left: Expr = self.comparison();
        if self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right: Expr = self.comparison();
            return Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        left
    }

    fn comparison(&mut self) -> Expr {
        let left: Expr = self.term();
        if self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right: Expr = self.term();
            return Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        left
    }

    fn term(&mut self) -> Expr {
        let mut left: Expr = self.factor();
        while self.match_tokens(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right: Expr = self.term();
            return Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        left
    }

    fn factor(&mut self) -> Expr {
        let mut left: Expr = self.unary();
        while self.match_tokens(&[TokenType::Slash, TokenType::Star, TokenType::Modulo]) {
            let operator = self.previous().clone();
            let right: Expr = self.unary();
            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        left
    }

    fn unary(&mut self) -> Expr {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let e = self.unary();
            return Expr::Unary {
                operator,
                expression: Box::new(e),
            };
        }
        self.call()
    }

    fn call(&mut self) -> Expr {
        let expr = self.primary();
        if self.match_tokens(&[TokenType::LeftParen]) {
            if !matches!(expr, Expr::Variable(_)) {
                self.error("undefined function call");
            }
            if self.match_tokens(&[TokenType::RightParen]) {
                return Expr::Call {
                    callee: Box::new(expr),
                    arguments: Vec::new(),
                };
            }
            let mut vec: Vec<Expr> = Vec::new();
            vec.push(self.unary()); //todo: change to self.logical()
            while self.match_tokens(&[TokenType::Comma]) {
                vec.push(self.unary()); //todo: change to self.logical()
            }
            self.consume(TokenType::RightParen, "call must end with a \")\"");
            return Expr::Call {
                callee: Box::new(expr),
                arguments: vec,
            };
        }
        expr
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
            return Expr::Literal(Value::String(match self.previous().literal.clone()
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
            return Expr::Variable(self.previous().clone());
        }
        if self.match_tokens(&[TokenType::LeftParen]) {
            let expression = self.logical();
            self.consume(
                TokenType::RightParen,
                "expected \")\" after expression u piece of shit",
            );
            return Expr::Grouping(Box::new(expression));
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

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    fn is_at_end(&self) -> bool {
        self.peek().tt == TokenType::Eof
    }
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
    fn error(&self, msg: &str) {
        KeyScriptError::error(
            KeyScriptError::ParserError,
            Some(msg),
            Some(self.peek().line),
            Some(self.filename));
    }
    fn consume(&mut self, t_type: TokenType, msg: &str) -> &Token {
        if self.peek().tt == t_type {
            return self.advance();
        }
        self.error(msg);
        panic!("kys");
    }
}