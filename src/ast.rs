use crate::scanner::{Token, TokenType, Value};

#[derive(Clone, Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
        line: usize,
    }, //assignment
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
        line: usize,
    }, //binary operations (+, -, .., ==, !=, <=, ..)
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
        line: usize,
    },
    Grouping(Box<Expr>), // "(" expression ")"
    Literal {
        val: Value,
        line: usize,
    },
    Unary {
        operator: Token,
        expression: Box<Expr>,
        line: usize,
    }, // ! or - (negate)
    Variable {
        name: Token,
        line: usize
    },
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Print {
        expr: Expr,
        line: usize,
    },
    Block {
        stmts: Vec<Stmt>,
        vars: Vec<TokenType>,
    },
    Expression(Expr),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
        line: usize,
    },
    Var {
        name: Token,
        value: Option<Expr>,
        t: TokenType,
        line: usize,
    },
    While {
        condition: Expr,
        block: Box<Stmt>,
        line: usize,
    },
    Fn {
        name: String,
        params: Vec<(TokenType, Token)>,
        body: Box<Stmt>,
        return_type: TokenType,
        line: usize,
    },
    Return {
        returnee: Expr,
        return_type: TokenType,
        line: usize,
    },
    Break (i32),
}
