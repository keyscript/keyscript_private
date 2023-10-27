use crate::ast::{Expr, Stmt};
use crate::errors::KeyScriptError;
use crate::scanner::{Token, TokenType, Value};
pub struct Parser<'a> {
    pub tokens: Vec<Token>,
    current: usize,
    filename: &'a str,
    pub vars: Vec<TokenType>,
    pub return_type: TokenType,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, filename: &'a str) -> Parser {
        Parser {
            tokens,
            current: 0,
            filename,
            vars: Vec::new(),
            return_type: TokenType::Void,
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            let decl = self.declaration();
            match decl {
                Stmt::Fn { name, params, body, return_type, line } => {
                    statements.insert(0, Stmt::Fn { name, params, body, return_type, line });
                }
                _ => statements.push(decl),
            }
        }
        statements
    }

    fn declaration(&mut self) -> Stmt {
        if self.match_tokens(&[TokenType::Bool, TokenType::Int, TokenType::Float, TokenType::String, TokenType::Void]) {
            let t = self.previous().clone();
            self.consume(TokenType::Identifier, "expected identifier after type declaration");
            let name = self.previous().clone();
            if self.match_tokens(&[TokenType::LeftParen]) {
                if t.tt == TokenType::String {
                    self.error("functions cannot return a string, use the main script or a void function");
                }
                if let Value::String(n) = name.literal.clone().unwrap() {
                    if n == "main" {
                        self.error("cant have a function called main, because the main script is called main");
                    }
                }
                return self.fn_decl(name, t.tt);
            } else {
                if self.match_tokens(&[TokenType::Void]) {
                    self.error("cannot have a variable of type void");
                }
                self.vars.push(t.tt);
                return self.var_decl(name, t)
            }
        }
        self.statement(-1)
    }

    fn block_declaration(&mut self, vars: &mut Vec<TokenType>, is_loop: i32) -> Stmt {
        if self.check(&TokenType::Break) {
            if is_loop == -1 {
                self.error("cannot have a break statement outside of a loop");
            }
            self.advance();
            self.consume(TokenType::Semicolon, "expected \";\" after break statement");
            return Stmt::Break(is_loop);
        }
        if self.match_tokens(&[TokenType::Bool, TokenType::Int, TokenType::Float, TokenType::String]) {
            let t = self.previous().clone();
            self.consume(TokenType::Identifier, "expected identifier after type declaration");
            let name = self.previous().clone();
            if self.match_tokens(&[TokenType::LeftParen]) {
                self.error("cannot have a function declaration inside a block");
                panic!("kys")
            } else {
                vars.push(t.tt);
                return self.var_decl(name, t)
            }
        }
        self.statement(is_loop)
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
        let body: Box<Stmt>;
        body = Box::new(self.block(Some(return_type), -1));
        Stmt::Fn {
            name: match name.literal {
                Some(Value::String(s)) => s,
                _ => panic!("unreachable?"),
            },
            params,
            body,
            return_type,
            line: name.line,
        }
    }

    fn var_decl(&mut self, name: Token, t: Token) -> Stmt {
        let value = if self.match_tokens(&[TokenType::Equal]) {
            Some(self.logical())
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "expected \";\" after variable declaration");
        let line = name.line;
        Stmt::Var {
            name,
            value,
            t: t.tt,
            line,
        }
    }

    fn statement(&mut self, is_loop: i32) -> Stmt {
        if self.match_tokens(&[TokenType::Print]) {
            if is_loop >= 0 {
                self.error("print inside loops is a feature that is not yet implemented, sorry :(");
            }
            return self.print_stmt();
        }
        if self.match_tokens(&[TokenType::Return]) {
            return self.return_stmt();
        }
        if self.match_tokens(&[TokenType::If]) {
            return self.if_stmt(is_loop);
        }
        if self.match_tokens(&[TokenType::While]) {
            return self.while_stmt();
        }
        self.expr_stmt()
    }

    fn print_stmt(&mut self) -> Stmt {
        let expr = self.logical(); // only can have binary(+) with primary
        self.consume(TokenType::Semicolon, "expected \";\" after print statement");
        Stmt::Print {
            expr,
            line: self.previous().line,
        }
    }

    fn parse_print(&mut self) -> Expr {
        let mut left: Expr = self.primary();
        while self.match_tokens(&[TokenType::Plus]) {
            let operator = self.previous().clone();
            let right: Expr = self.parse_print();
            let line = operator.line;
            return Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                line,
            };
        }
        self.primary()
    }

    fn return_stmt(&mut self) -> Stmt {
        if self.match_tokens(&[TokenType::Semicolon]) {
            self.error("expected expression after return statement");
        }
        let value = self.logical();
        self.consume(TokenType::Semicolon, "expected \";\" after return statement");
        Stmt::Return{returnee: value, return_type: self.return_type, line: self.previous().line}
    }

    fn if_stmt(&mut self, is_loop: i32) -> Stmt {
        let condition = self.logical();
        let then_branch = Box::new(self.block(None, if is_loop != -1 {is_loop + 1} else {-1}));
        let else_branch = if self.match_tokens(&[TokenType::Else]) {
            Some(Box::new(self.block(None, is_loop)))
        } else {
            None
        };
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            line: self.previous().line,
        }
    }

    fn while_stmt(&mut self) -> Stmt {
        let condition = self.logical();
        let block = Box::new(self.block(None, 0));
        Stmt::While {
            condition,
            block,
            line: self.previous().line,
        }
    }

    fn block(&mut self, enforce_return_type: Option<TokenType>, is_loop: i32) -> Stmt {
        self.consume(TokenType::LeftBrace, "block must start with a \"{\"");
        let mut had_return: bool = false;
        if enforce_return_type.is_some() {
            self.return_type = enforce_return_type.unwrap();
        }
        let mut vars1: Vec<TokenType> = Vec::new();
        let mut stmts: Vec<Stmt> = Vec::new();
        while !self.is_at_end() && !self.check(&TokenType::RightBrace) {
            let stmt = self.block_declaration(&mut vars1, is_loop);
            match stmt.clone() {
                Stmt::If {then_branch, else_branch, ..} => {
                    match *then_branch.clone() {
                        Stmt::Block {vars, ..} => {
                            for var in vars {
                                vars1.push(var);
                            }
                        }
                        _ => {}
                    }
                    if let Some(else_branch) = else_branch {
                        match *else_branch {
                            Stmt::Block {vars, ..} => {
                                for var in vars {
                                    vars1.push(var);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Stmt::While {block, ..} => {
                    match *block {
                        Stmt::Block {vars, ..} => {
                            for var in vars {
                                vars1.push(var);
                            }
                        }
                        _ => {}
                    }
                }
                Stmt::Block {vars, ..} => {
                    for var in vars {
                        vars1.push(var);
                    }
                }
                _ => {}
            }
            stmts.push(stmt);
            if let Stmt::Return {..} = stmts[stmts.len() - 1] {
                had_return = true;
            }
        }
        if let Some(TokenType::Void) = enforce_return_type {
            if had_return {
                self.error("void functions cannot return a value");
            }
        } else if !enforce_return_type.is_none() {
            if !had_return {
                self.error("non void functions must return a value");
            }
        }
        if enforce_return_type.is_some() {
            self.return_type = TokenType::Void;
        }
        self.consume(TokenType::RightBrace, "block must end with a \"}\"");
        Stmt::Block{stmts, vars: vars1}
    }

    fn expr_stmt(&mut self) -> Stmt {
        let expr = self.assignment();
        if let Expr::Variable{..} = expr {
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
                Expr::Variable{name, line} => {
                    match self.previous().tt {
                        TokenType::Equal => {
                            return Expr::Assign {
                                name,
                                value: Box::new(self.logical()),
                                line,
                            }
                        }
                        TokenType::PlusEqual | TokenType::MinusEqual | TokenType::StarEqual | TokenType::SlashEqual => {
                            return Expr::Assign {
                                name: name.clone(),
                                value: Box::new(Expr::Binary {
                                    left: Box::new(Expr::Variable{name, line}),
                                    operator: Token {
                                        tt: match self.previous().tt {
                                            TokenType::PlusEqual => TokenType::Plus,
                                            TokenType::MinusEqual => TokenType::Minus,
                                            TokenType::StarEqual => TokenType::Star,
                                            TokenType::SlashEqual => TokenType::Slash,
                                            _ => panic!("unreachable?"),
                                        },
                                        literal: None,
                                        line,
                                    },
                                    right: Box::new(self.logical()),
                                    line,
                                }),
                                line,
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
            let line = operator.line;
            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                line,
            };
        }
        left
    }

    fn equality(&mut self) -> Expr {
        let left: Expr = self.comparison();
        if self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right: Expr = self.comparison();
            let line = operator.line;
            return Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                line,
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
            let line = operator.line;
            return Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                line,
            };
        }
        left
    }

    fn term(&mut self) -> Expr {
        let left: Expr = self.factor();
        while self.match_tokens(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right: Expr = self.term();
            let line = operator.line;
            return Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                line,
            };
        }
        left
    }

    fn factor(&mut self) -> Expr {
        let mut left: Expr = self.unary();
        while self.match_tokens(&[TokenType::Slash, TokenType::Star, TokenType::Modulo]) {
            let operator = self.previous().clone();
            let right: Expr = self.unary();
            let line = operator.line;
            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                line
            };
        }
        left
    }

    fn unary(&mut self) -> Expr {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let e = self.unary();
            let line = operator.line;
            return Expr::Unary {
                operator,
                expression: Box::new(e),
                line,
            };
        }
        self.call()
    }

    fn call(&mut self) -> Expr {
        let expr = self.primary();
        let line = self.previous().line;
        if self.match_tokens(&[TokenType::LeftParen]) {
            if !matches!(expr, Expr::Variable{..}) {
                self.error("undefined function call");
            }
            if self.match_tokens(&[TokenType::RightParen]) {
                return Expr::Call {
                    callee: Box::new(expr),
                    arguments: Vec::new(),
                    line,
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
                line,
            };
        }
        expr
    }

    fn primary(&mut self) -> Expr {
        if self.match_tokens(&[TokenType::Value]) {
            match self.previous().clone().literal {
                Some(Value::Bool(b)) => return Expr::Literal{val: Value::Bool(b), line: self.previous().line},
                Some(Value::Int(n)) => return Expr::Literal{val: Value::Int(n), line: self.previous().line},
                Some(Value::Float(n)) => return Expr::Literal{val: Value::Float(n), line: self.previous().line},
                Some(Value::String(s)) => return Expr::Literal{val: Value::String(s), line: self.previous().line},
                _ => panic!("kys"),
            }
        }
        if self.match_tokens(&[TokenType::Identifier]) {
            return Expr::Variable{name: self.previous().clone(), line: self.previous().line};
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