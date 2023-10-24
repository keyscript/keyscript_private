use crate::ast::{Expr, Stmt};
use crate::errors::KeyScriptError;
use crate::scanner::{Token, TokenType, Value};
pub struct Parser<'a> {
    pub tokens: Vec<Token>,
    current: usize,
    filename: &'a str,
    pub vars: Vec<TokenType>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, filename: &'a str) -> Parser {
        Parser {
            tokens,
            current: 0,
            filename,
            vars: Vec::new()
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            let decl = self.declaration();
            match decl {
                Stmt::Fn { name, params, body, return_type } => {
                    statements.insert(0, Stmt::Fn { name, params, body, return_type });
                }
                _ => statements.push(decl),
            }
        }
        statements
    }

    fn declaration(&mut self) -> Stmt {
        if self.match_tokens(&[TokenType::Bool, TokenType::Int, TokenType::Float, TokenType::String]) {
            let t = self.previous().clone();
            self.consume(TokenType::Identifier, "expected identifier after type declaration");
            let name = self.previous().clone();
            if self.match_tokens(&[TokenType::LeftParen]) {
                return self.fn_decl(name, t.tt);
            } else {
                self.vars.push(t.tt);
                return self.var_decl(name, t)
            }
        }
        self.statement()
    }

    fn block_declaration(&mut self, vars: &mut Vec<TokenType>) -> Stmt {
        if self.match_tokens(&[TokenType::Bool, TokenType::Int, TokenType::Float, TokenType::String]) {
            let t = self.previous().clone();
            self.consume(TokenType::Identifier, "expected identifier after type declaration");
            let name = self.previous().clone();
            if self.match_tokens(&[TokenType::LeftParen]) {
                return self.fn_decl(name, t.tt);
            } else {
                vars.push(t.tt);
                return self.var_decl(name, t)
            }
        }
        self.statement()
    }

    fn fn_decl(&mut self, name: Token, return_type: TokenType) -> Stmt {
        let mut params: Vec<(TokenType, Token)> = Vec::new();
        if self.match_tokens(&[TokenType::Bool, TokenType::Int, TokenType::Float, TokenType::String]) {
            let t = self.previous().tt;
            let identifier = self.consume(TokenType::Identifier, "expected identifier after type declaration");
            params.push((t, identifier.clone()));
        }
        while !self.check(&TokenType::RightParen) {
            self.consume(TokenType::Comma, "expected \",\" after identifier");
            if !self.match_tokens(&[TokenType::Bool, TokenType::Int, TokenType::Float, TokenType::String]) {
                self.error("expected type declaration after comma");
            }
            let t = self.previous().tt;
            let identifier = self.consume(TokenType::Identifier, "expected identifier after type declaration");
            params.push((t, identifier.clone()));
        }
        self.consume(TokenType::RightParen, "expected \")\" after function declaration");
        let body = Box::new(self.block());
        Stmt::Fn {
            name: match name.literal {
                Some(Value::String(s)) => s,
                _ => panic!("unreachable?"),
            },
            params,
            body,
            return_type,
        }
    }

    fn var_decl(&mut self, name: Token, t: Token) -> Stmt {
        let value = if self.match_tokens(&[TokenType::Equal]) {
            Some(self.logical())
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "expected \";\" after variable declaration");
        Stmt::Var {
            name,
            value,
            t: t.tt,
        }
    }

    fn statement(&mut self) -> Stmt {
        if self.match_tokens(&[TokenType::Print]) {
            return self.print_stmt();
        }
        if self.match_tokens(&[TokenType::Return]) {
            return self.return_stmt();
        }
        if self.match_tokens(&[TokenType::If]) {
            return self.if_stmt();
        }
        if self.match_tokens(&[TokenType::While]) {
            return self.while_stmt();
        }
        self.expr_stmt()
    }

    fn print_stmt(&mut self) -> Stmt {
        let expr = self.logical();
        self.consume(TokenType::Semicolon, "expected \";\" after print statement");
        Stmt::Print(expr)
    }

    fn return_stmt(&mut self) -> Stmt {
        if self.match_tokens(&[TokenType::Semicolon]) {
            self.error("expected expression after return statement");
        }
        let value = self.logical();
        self.consume(TokenType::Semicolon, "expected \";\" after return statement");
        Stmt::Return(value)
    }

    fn if_stmt(&mut self) -> Stmt {
        let condition = self.logical();
        let then_branch = Box::new(self.block());
        let else_branch = if self.match_tokens(&[TokenType::Else]) {
            Some(Box::new(self.block()))
        } else {
            None
        };
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        }
    }

    fn while_stmt(&mut self) -> Stmt {
        let condition = self.logical();
        let block = Box::new(self.block());
        Stmt::While {
            condition,
            block,
        }
    }

    fn block(&mut self) -> Stmt {
        self.consume(TokenType::LeftBrace, "block must start with a \"{\"");
        let mut vars: Vec<TokenType> = Vec::new();
        let mut stmts: Vec<Stmt> = Vec::new();
        while !self.is_at_end() && !self.check(&TokenType::RightBrace) {
            stmts.push(self.block_declaration(&mut vars));
        }
        self.consume(TokenType::RightBrace, "block must end with a \"}\"");
        Stmt::Block{stmts, vars}
    }

    fn expr_stmt(&mut self) -> Stmt {
        let expr = self.assignment();
        if let Expr::Variable(_) = expr {
            self.error("cannot access a variable on its own, use it in an expression");
        }
        let stmt = Stmt::Expression(expr);
        self.consume(TokenType::Semicolon, "missing ; at the end of the line");
        stmt
    }

    fn assignment(&mut self) -> Expr {
        let identifier = self.logical();
        if self.match_tokens(&[TokenType::Equal, TokenType::PlusEqual, TokenType::MinusEqual, TokenType::StarEqual, TokenType::SlashEqual]) {
            match identifier {
                Expr::Variable(name) => {
                    match self.previous().tt {
                        TokenType::Equal => {
                            return Expr::Assign {
                                name,
                                value: Box::new(self.logical()),
                            }
                        }
                        TokenType::PlusEqual | TokenType::MinusEqual | TokenType::StarEqual | TokenType::SlashEqual => {
                            return Expr::Assign {
                                name: name.clone(),
                                value: Box::new(Expr::Binary {
                                    left: Box::new(Expr::Variable(name)),
                                    operator: Token {
                                        tt: match self.previous().tt {
                                            TokenType::PlusEqual => TokenType::Plus,
                                            TokenType::MinusEqual => TokenType::Minus,
                                            TokenType::StarEqual => TokenType::Star,
                                            TokenType::SlashEqual => TokenType::Slash,
                                            _ => panic!("unreachable?"),
                                        },
                                        literal: None,
                                        line: 0,
                                    },
                                    right: Box::new(self.logical()),
                                }),
                            }
                        }
                        _ => {
                            self.error("value incorrect");
                            panic!();
                        }
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

    fn logical(&mut self) -> Expr {
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
        let left: Expr = self.factor();
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
            vec.push(self.logical());
            while self.match_tokens(&[TokenType::Comma]) {
                vec.push(self.logical());
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
        if self.match_tokens(&[TokenType::Value]) {
            match self.previous().clone().literal {
                Some(Value::Bool(b)) => return Expr::Literal(Value::Bool(b)),
                Some(Value::Int(n)) => return Expr::Literal(Value::Int(n)),
                Some(Value::Float(n)) => return Expr::Literal(Value::Float(n)),
                Some(Value::String(s)) => return Expr::Literal(Value::String(s)),
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